use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::path::Path;

use serde_json::Value;

use super::ssts::SSTS;
use super::validation::ssts_from_payload_owned;

pub fn load_ssts_json(path: &Path, require_ordering: bool) -> Result<SSTS> {
    let file = File::open(path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("load_ssts_json: failed to open {}: {e}", path.display()),
        )
    })?;
    let mut reader = BufReader::new(file);

    let payload: Value = serde_json::from_reader(&mut reader).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("load_ssts_json: invalid json {}: {e}", path.display()),
        )
    })?;

    ssts_from_payload_owned(payload, require_ordering).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!(
                "load_ssts_json: invalid ssts payload {}: {e}",
                path.display()
            ),
        )
    })
}

pub fn save_ssts_json(ssts: &SSTS, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|e| {
            Error::new(
                e.kind(),
                format!("save_ssts_json: failed to create {}: {e}", parent.display()),
            )
        })?;
    }

    let mut file = File::create(path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("save_ssts_json: failed to create {}: {e}", path.display()),
        )
    })?;
    let mut writer = BufWriter::new(&mut file);

    serde_json::to_writer_pretty(&mut writer, ssts).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!(
                "save_ssts_json: serialization failed {}: {e}",
                path.display()
            ),
        )
    })?;

    writer.flush().map_err(|e| {
        Error::new(
            e.kind(),
            format!("save_ssts_json: failed to write {}: {e}", path.display()),
        )
    })
}
