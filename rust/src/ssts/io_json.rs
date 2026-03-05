use std::fs::{create_dir_all, read_to_string, File};
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

use serde_json::Value;

use super::ssts::SSTS;
use super::validation::ssts_from_payload;

pub fn load_ssts_json(path: &Path, require_ordering: bool) -> Result<SSTS> {
    let raw = read_to_string(path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("load_ssts_json: failed to read {}: {e}", path.display()),
        )
    })?;

    let payload: Value = serde_json::from_str(&raw).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("load_ssts_json: invalid json {}: {e}", path.display()),
        )
    })?;

    ssts_from_payload(&payload, require_ordering).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("load_ssts_json: invalid ssts payload {}: {e}", path.display()),
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

    let payload = ssts.to_payload();
    let json = serde_json::to_string_pretty(&payload).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("save_ssts_json: serialization failed {}: {e}", path.display()),
        )
    })?;

    let mut file = File::create(path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("save_ssts_json: failed to create {}: {e}", path.display()),
        )
    })?;

    file.write_all(json.as_bytes()).map_err(|e| {
        Error::new(
            e.kind(),
            format!("save_ssts_json: failed to write {}: {e}", path.display()),
        )
    })
}
