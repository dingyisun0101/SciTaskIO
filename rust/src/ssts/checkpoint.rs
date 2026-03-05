use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::io_json::load_ssts_json;
use super::ssts::SSTS;

#[derive(Debug, Clone)]
pub struct CheckpointPreflightReport {
    pub scanned_files: usize,
    pub valid_files: usize,
    pub removed_invalid_files: usize,
}

fn parse_checkpoint_epoch(path: &Path) -> Option<usize> {
    if path.extension().and_then(|s| s.to_str()) != Some("json") {
        return None;
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<usize>().ok())
}

pub fn list_checkpoint_files(checkpoint_dir: &Path) -> io::Result<Vec<(usize, PathBuf)>> {
    let mut items = Vec::new();
    let read_dir = match fs::read_dir(checkpoint_dir) {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(items),
        Err(err) => return Err(err),
    };

    for entry in read_dir {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let Some(epoch) = parse_checkpoint_epoch(&path) else {
            continue;
        };
        items.push((epoch, path));
    }

    items.sort_by_key(|(epoch, _)| *epoch);
    Ok(items)
}

pub fn latest_checkpoint_path(checkpoint_dir: &Path) -> io::Result<Option<(usize, PathBuf)>> {
    let items = list_checkpoint_files(checkpoint_dir)?;
    Ok(items.into_iter().next_back())
}

pub fn preflight_checkpoint_dir(
    checkpoint_dir: &Path,
    require_ordering: bool,
    remove_invalid: bool,
) -> io::Result<CheckpointPreflightReport> {
    let items = list_checkpoint_files(checkpoint_dir)?;
    let mut valid_files = 0usize;
    let mut removed_invalid_files = 0usize;

    for (_, path) in &items {
        let is_valid = match fs::metadata(path) {
            Ok(meta) if meta.len() == 0 => false,
            Ok(_) => load_ssts_json(path, require_ordering).is_ok(),
            Err(_) => false,
        };

        if is_valid {
            valid_files += 1;
            continue;
        }

        if remove_invalid && path.exists() {
            fs::remove_file(path)?;
            removed_invalid_files += 1;
        }
    }

    Ok(CheckpointPreflightReport {
        scanned_files: items.len(),
        valid_files,
        removed_invalid_files,
    })
}

pub fn load_latest_checkpoint(
    checkpoint_dir: &Path,
    require_ordering: bool,
    remove_invalid: bool,
) -> io::Result<Option<(usize, SSTS)>> {
    let items = list_checkpoint_files(checkpoint_dir)?;

    for (epoch, path) in items.into_iter().rev() {
        let meta = match fs::metadata(&path) {
            Ok(meta) => meta,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };

        if meta.len() == 0 {
            if remove_invalid && path.exists() {
                fs::remove_file(&path)?;
                continue;
            }
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("empty checkpoint file: {}", path.display()),
            ));
        }

        match load_ssts_json(&path, require_ordering) {
            Ok(ssts) => return Ok(Some((epoch, ssts))),
            Err(err) => {
                if remove_invalid && path.exists() {
                    fs::remove_file(&path)?;
                    continue;
                }
                return Err(Error::new(
                    err.kind(),
                    format!("invalid checkpoint {}: {err}", path.display()),
                ));
            }
        }
    }

    Ok(None)
}

pub fn prune_newer_than(checkpoint_dir: &Path, keep_epoch: usize) -> io::Result<usize> {
    let items = list_checkpoint_files(checkpoint_dir)?;
    let mut removed = 0usize;

    for (epoch, path) in items {
        if epoch <= keep_epoch {
            continue;
        }
        if path.exists() {
            fs::remove_file(path)?;
            removed += 1;
        }
    }

    Ok(removed)
}
