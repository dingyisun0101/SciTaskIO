#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::ssts::{load_ssts_json, SSTSSeries};

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("schema")
            .join("fixtures")
            .join("contract")
            .join(path)
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
}
