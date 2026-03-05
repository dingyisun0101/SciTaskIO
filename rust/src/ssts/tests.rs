#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::{Map, Value};

    use crate::ssts::{
        SSTSBuilder, SSTSSeries, decode_vec, load_ssts_json, save_ssts_json, sync_checkpoint_dirs,
    };

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("schema")
            .join("fixtures")
            .join("contract")
            .join(path)
    }

    fn tmp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("sci_task_io_{prefix}_{nonce}"));
        create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn minimal_ssts_with_epoch(epoch: usize) -> Value {
        let mut metadata = Map::new();
        metadata.insert(
            "series_id".to_string(),
            Value::String(format!("run_{epoch}")),
        );

        serde_json::json!({
            "metadata": metadata,
            "signals": {
                "track_1": {
                    "label": "frequencies",
                    "times": [epoch],
                    "signal": [[0.5, 0.5]]
                }
            }
        })
    }

    #[test]
    fn load_valid_fixture() {
        let path = fixture("valid/minimal.json");
        let ssts = load_ssts_json(&path, false).expect("valid fixture should load");
        assert_eq!(ssts.tracks.len(), 1);
        assert_eq!(ssts.tracks[0].label, "frequencies");
    }

    #[test]
    fn reject_invalid_track_key() {
        let path = fixture("invalid/bad_track_key.json");
        let err = load_ssts_json(&path, false).expect_err("invalid fixture must fail");
        assert!(err.to_string().contains("invalid track key"));
    }

    #[test]
    fn reject_length_mismatch() {
        let path = fixture("invalid/times_signal_len_mismatch.json");
        let err = load_ssts_json(&path, false).expect_err("len mismatch must fail");
        assert!(err.to_string().contains("lengths must match"));
    }

    #[test]
    fn series_load_and_sort() {
        let dir = fixture("valid");
        let mut series = SSTSSeries::new(dir, false);
        series.load().expect("series should load");
        assert_eq!(series.entries.len(), 3);

        let ids = series.serial_ids().expect("serial ids should be available");
        assert_eq!(ids, vec!["minimal", "run_A", "run_B"]);
    }

    #[test]
    fn builder_and_decode_helpers_work_for_sparse_tracks() {
        let mut builder = SSTSBuilder::new();
        builder
            .push_row("frequencies", 0usize, vec![0.5f64, 0.5f64])
            .expect("row should serialize");
        builder
            .push_row(
                "space",
                0usize,
                vec![vec![1usize, 0usize], vec![2usize, 1usize]],
            )
            .expect("row should serialize");
        builder
            .push_row("frequencies", 10usize, vec![0.6f64, 0.4f64])
            .expect("row should serialize");

        let ssts = builder.build();
        let frequencies = ssts
            .track_by_label("frequencies")
            .expect("frequencies track should exist");
        assert_eq!(frequencies.len(), 2);
        let (last_time, _) = frequencies.latest_row().expect("latest row should exist");
        assert_eq!(last_time, &Value::from(10usize));

        let space = ssts
            .track_by_label("space")
            .expect("space track should exist");
        assert_eq!(space.len(), 1);

        let decoded_times: Vec<usize> =
            decode_vec(frequencies.times()).expect("times should decode as usize");
        assert_eq!(decoded_times, vec![0usize, 10usize]);
        let decoded_signal: Vec<Vec<f64>> =
            decode_vec(frequencies.signal()).expect("signal should decode as Vec<Vec<f64>>");
        assert_eq!(decoded_signal.len(), 2);
    }

    #[test]
    fn sync_checkpoint_dirs_prunes_to_latest_shared_epoch() {
        let root = tmp_dir("sync");
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        create_dir_all(&dir_a).expect("dir a should exist");
        create_dir_all(&dir_b).expect("dir b should exist");

        for epoch in [1usize, 2usize] {
            let payload = minimal_ssts_with_epoch(epoch);
            let ssts =
                crate::ssts::ssts_from_payload(&payload, false).expect("payload should parse");
            save_ssts_json(&ssts, &dir_a.join(format!("{epoch}.json")))
                .expect("checkpoint should save");
        }
        for epoch in [1usize, 3usize] {
            let payload = minimal_ssts_with_epoch(epoch);
            let ssts =
                crate::ssts::ssts_from_payload(&payload, false).expect("payload should parse");
            save_ssts_json(&ssts, &dir_b.join(format!("{epoch}.json")))
                .expect("checkpoint should save");
        }

        let report = sync_checkpoint_dirs(&[dir_a.clone(), dir_b.clone()], false, true, true)
            .expect("sync should succeed");

        assert_eq!(report.keep_epoch, Some(1usize));
        assert_eq!(report.removed_newer_files, 2usize);
        assert!(report.out_of_sync);

        let remaining_a =
            crate::ssts::list_checkpoint_files(&dir_a).expect("checkpoints should list");
        let remaining_b =
            crate::ssts::list_checkpoint_files(&dir_b).expect("checkpoints should list");
        assert_eq!(
            remaining_a
                .iter()
                .map(|(epoch, _)| *epoch)
                .collect::<Vec<_>>(),
            vec![1usize]
        );
        assert_eq!(
            remaining_b
                .iter()
                .map(|(epoch, _)| *epoch)
                .collect::<Vec<_>>(),
            vec![1usize]
        );

        remove_dir_all(root).expect("temp root should be removed");
    }
}
