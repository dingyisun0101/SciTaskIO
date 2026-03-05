use std::path::PathBuf;

use sci_task_io::ssts::{SSTS, SignalTrack};
use serde_json::{Map, Value};

fn main() -> std::io::Result<()> {
    // Build one track.
    let track = SignalTrack {
        label: "frequencies".to_string(),
        times: vec![Value::from(0), Value::from(10), Value::from(20)],
        signal: vec![
            Value::Array(vec![Value::from(0.5), Value::from(0.5)]),
            Value::Array(vec![Value::from(0.6), Value::from(0.4)]),
            Value::Array(vec![Value::from(0.55), Value::from(0.45)]),
        ],
        extra: Map::new(),
    };

    // Build top-level metadata/scalars.
    let mut metadata = Map::new();
    metadata.insert("series_id".to_string(), Value::from("demo_series"));
    metadata.insert(
        "source".to_string(),
        Value::from("rust/examples/basic_usage.rs"),
    );

    let mut scalars = Map::new();
    scalars.insert("seed".to_string(), Value::from(123));
    scalars.insert("dt".to_string(), Value::from(0.01));

    let ssts = SSTS {
        tracks: vec![track],
        metadata,
        scalars,
        extra: Map::new(),
    };

    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("output")
        .join("demo_series.json");

    ssts.save_json(&output_path)?;
    println!("wrote {}", output_path.display());

    let loaded = SSTS::from_json(&output_path, false)?;
    println!("loaded tracks: {}", loaded.tracks.len());
    println!("first label: {}", loaded.tracks[0].label);

    Ok(())
}
