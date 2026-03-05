use std::collections::BTreeSet;
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

#[derive(Debug, Clone)]
pub struct CheckpointSyncReport {
    pub scanned_dirs: usize,
    pub dirs_with_checkpoints: usize,
    pub keep_epoch: Option<usize>,
    pub removed_invalid_files: usize,
    pub removed_newer_files: usize,
    pub out_of_sync: bool,
}

#[derive(Debug, Clone)]
struct CheckpointScan {
    checkpoint_files: Vec<(usize, PathBuf)>,
    epochs: BTreeSet<usize>,
    invalid_files: Vec<PathBuf>,
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

fn scan_checkpoint_dir(
    checkpoint_dir: &Path,
    require_ordering: bool,
) -> io::Result<CheckpointScan> {
    let mut checkpoint_files = Vec::new();
    let mut epochs = BTreeSet::new();
    let mut invalid_files = Vec::new();

    for (epoch, path) in list_checkpoint_files(checkpoint_dir)? {
        let is_valid = match fs::metadata(&path) {
            Ok(meta) if meta.len() == 0 => false,
            Ok(_) => load_ssts_json(&path, require_ordering).is_ok(),
            Err(_) => false,
        };

        if is_valid {
            checkpoint_files.push((epoch, path));
            epochs.insert(epoch);
        } else {
            invalid_files.push(path);
        }
    }

    Ok(CheckpointScan {
        checkpoint_files,
        epochs,
        invalid_files,
    })
}

pub fn sync_checkpoint_dirs(
    checkpoint_dirs: &[PathBuf],
    require_ordering: bool,
    remove_invalid: bool,
    prune_newer: bool,
) -> io::Result<CheckpointSyncReport> {
    let mut scans = Vec::new();
    for checkpoint_dir in checkpoint_dirs {
        if !checkpoint_dir.is_dir() {
            continue;
        }
        scans.push(scan_checkpoint_dir(checkpoint_dir, require_ordering)?);
    }

    let scanned_dirs = scans.len();
    let dirs_with_checkpoints = scans.iter().filter(|scan| !scan.epochs.is_empty()).count();
    let mut removed_invalid_files = 0usize;
    if remove_invalid {
        for scan in &scans {
            for invalid_path in &scan.invalid_files {
                if invalid_path.exists() {
                    fs::remove_file(invalid_path)?;
                    removed_invalid_files += 1;
                }
            }
        }
    }

    if dirs_with_checkpoints <= 1 {
        let keep_epoch = scans
            .iter()
            .filter_map(|scan| scan.epochs.iter().next_back().copied())
            .min();
        return Ok(CheckpointSyncReport {
            scanned_dirs,
            dirs_with_checkpoints,
            keep_epoch,
            removed_invalid_files,
            removed_newer_files: 0,
            out_of_sync: false,
        });
    }

    let mut common_epochs: Option<BTreeSet<usize>> = None;
    for scan in scans.iter().filter(|scan| !scan.epochs.is_empty()) {
        if let Some(shared) = common_epochs.as_mut() {
            shared.retain(|epoch| scan.epochs.contains(epoch));
        } else {
            common_epochs = Some(scan.epochs.clone());
        }
    }

    let common_epochs = common_epochs.unwrap_or_default();
    let keep_epoch = match common_epochs.iter().next_back().copied() {
        Some(epoch) => epoch,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "checkpoint directories have no shared checkpoint epoch",
            ));
        }
    };

    let mut out_of_sync = false;
    for scan in &scans {
        if scan.epochs.is_empty() {
            continue;
        }
        let latest_epoch = scan
            .epochs
            .iter()
            .next_back()
            .copied()
            .unwrap_or(keep_epoch);
        if latest_epoch != keep_epoch {
            out_of_sync = true;
        }
    }

    let mut removed_newer_files = 0usize;
    if prune_newer {
        for scan in &scans {
            for (epoch, path) in &scan.checkpoint_files {
                if *epoch <= keep_epoch {
                    continue;
                }
                if path.exists() {
                    fs::remove_file(path)?;
                    removed_newer_files += 1;
                }
            }
        }
        if removed_newer_files > 0 {
            out_of_sync = true;
        }
    }

    Ok(CheckpointSyncReport {
        scanned_dirs,
        dirs_with_checkpoints,
        keep_epoch: Some(keep_epoch),
        removed_invalid_files,
        removed_newer_files,
        out_of_sync,
    })
}
