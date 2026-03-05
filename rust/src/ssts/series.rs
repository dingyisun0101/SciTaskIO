use std::fs::read_dir;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::io_json::load_ssts_json;
use super::ssts::SSTS;

#[derive(Clone, Debug)]
pub struct SeriesEntry {
    pub serial_id: String,
    pub path: PathBuf,
    pub ssts: SSTS,
}

#[derive(Clone, Debug)]
pub struct SSTSSeries {
    pub root_dir: PathBuf,
    pub require_ordering: bool,
    pub entries: Vec<SeriesEntry>,
}

impl SSTSSeries {
    pub fn new(root_dir: impl AsRef<Path>, require_ordering: bool) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            require_ordering,
            entries: Vec::new(),
        }
    }

    pub fn from_dir(root_dir: impl AsRef<Path>, require_ordering: bool) -> Result<Self> {
        let mut out = Self::new(root_dir, require_ordering);
        out.load()?;
        Ok(out)
    }

    pub fn load_dir(root_dir: impl AsRef<Path>, require_ordering: bool) -> Result<Self> {
        Self::from_dir(root_dir, require_ordering)
    }

    fn iter_series_paths(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        let entries = read_dir(&self.root_dir).map_err(|e| {
            Error::new(
                e.kind(),
                format!(
                    "ssts_series: failed to read {}: {e}",
                    self.root_dir.display()
                ),
            )
        })?;

        for entry in entries {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.ends_with(".scalars.json"))
            {
                continue;
            }
            out.push(path);
        }

        out.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(out)
    }

    fn resolve_serial_id(ssts: &SSTS, path: &Path) -> String {
        let serial_id = ssts.metadata.get("serial_id");
        if let Some(v) = serial_id {
            return match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            };
        }

        let series_id = ssts.metadata.get("series_id");
        if let Some(v) = series_id {
            return match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            };
        }

        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string()
    }

    pub fn load(&mut self) -> Result<()> {
        let paths = self.iter_series_paths()?;
        let mut entries = Vec::new();

        for path in paths {
            let ssts = load_ssts_json(&path, self.require_ordering)?;
            let serial_id = Self::resolve_serial_id(&ssts, &path);
            entries.push(SeriesEntry {
                serial_id,
                path,
                ssts,
            });
        }

        entries.sort_by(|a, b| a.serial_id.cmp(&b.serial_id));
        self.entries = entries;
        Ok(())
    }

    pub fn load_all(&mut self) -> Result<()> {
        self.load()
    }

    pub fn serial_ids(&mut self) -> Result<Vec<String>> {
        if self.entries.is_empty() {
            self.load()?;
        }
        Ok(self.entries.iter().map(|e| e.serial_id.clone()).collect())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn require_loaded(&self) -> Result<()> {
        if self.entries.is_empty() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "ssts_series: no entries loaded; call load() first",
            ));
        }
        Ok(())
    }
}
