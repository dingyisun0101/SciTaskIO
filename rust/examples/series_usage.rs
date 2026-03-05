use std::path::PathBuf;

use sci_task_io::ssts::SSTSSeries;

fn main() -> std::io::Result<()> {
    // Use shared contract fixtures as sample input.
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("schema")
        .join("fixtures")
        .join("contract")
        .join("valid");

    let mut series = SSTSSeries::from_dir(&fixture_dir, false)?;

    let ids = series.serial_ids()?;
    println!("fixture dir: {}", fixture_dir.display());
    println!("loaded entries: {}", series.entries.len());
    println!("serial ids: {:?}", ids);

    Ok(())
}
