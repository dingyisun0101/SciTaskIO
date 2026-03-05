use std::io::Result;
use std::path::Path;

use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct SignalTrack {
    pub label: String,
    pub times: Vec<Value>,
    pub signal: Vec<Value>,
    pub extra: Map<String, Value>,
}

impl SignalTrack {
    pub fn to_payload(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("label".to_string(), Value::String(self.label.clone()));
        obj.insert("times".to_string(), Value::Array(self.times.clone()));
        obj.insert("signal".to_string(), Value::Array(self.signal.clone()));
        for (k, v) in &self.extra {
            obj.insert(k.clone(), v.clone());
        }
        Value::Object(obj)
    }
}

#[derive(Clone, Debug)]
pub struct SSTS {
    pub tracks: Vec<SignalTrack>,
    pub metadata: Map<String, Value>,
    pub scalars: Map<String, Value>,
    pub extra: Map<String, Value>,
}

impl SSTS {
    pub fn to_payload(&self) -> Value {
        let mut signals = Map::new();
        for (idx, track) in self.tracks.iter().enumerate() {
            signals.insert(format!("track_{}", idx + 1), track.to_payload());
        }

        let mut payload = Map::new();
        payload.insert("metadata".to_string(), Value::Object(self.metadata.clone()));
        payload.insert("scalars".to_string(), Value::Object(self.scalars.clone()));
        payload.insert("signals".to_string(), Value::Object(signals));

        for (k, v) in &self.extra {
            payload.insert(k.clone(), v.clone());
        }

        Value::Object(payload)
    }

    pub fn from_json(path: &Path, require_ordering: bool) -> Result<Self> {
        super::io_json::load_ssts_json(path, require_ordering)
    }

    pub fn load_json(path: &Path, require_ordering: bool) -> Result<Self> {
        Self::from_json(path, require_ordering)
    }

    pub fn save_json(&self, path: &Path) -> Result<()> {
        super::io_json::save_ssts_json(self, path)
    }

    pub fn to_json_file(&self, path: &Path) -> Result<()> {
        self.save_json(path)
    }
}
