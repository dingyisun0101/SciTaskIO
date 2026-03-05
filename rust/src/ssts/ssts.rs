use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Result;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct SignalTrack {
    pub label: String,
    pub times: Vec<Value>,
    pub signal: Vec<Value>,
    pub extra: Map<String, Value>,
}

impl SignalTrack {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            times: Vec::new(),
            signal: Vec::new(),
            extra: Map::new(),
        }
    }

    pub fn push_row(&mut self, time: Value, signal: Value) {
        self.times.push(time);
        self.signal.push(signal);
    }

    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    pub fn latest_row(&self) -> Option<(&Value, &Value)> {
        let idx = self.times.len().checked_sub(1)?;
        Some((&self.times[idx], &self.signal[idx]))
    }

    pub fn times(&self) -> &[Value] {
        &self.times
    }

    pub fn signal(&self) -> &[Value] {
        &self.signal
    }

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
    pub fn empty() -> Self {
        Self {
            tracks: Vec::new(),
            metadata: Map::new(),
            scalars: Map::new(),
            extra: Map::new(),
        }
    }

    pub fn push_track(&mut self, track: SignalTrack) {
        self.tracks.push(track);
    }

    pub fn track_by_label(&self, label: &str) -> Option<&SignalTrack> {
        self.tracks.iter().find(|t| t.label == label)
    }

    pub fn tracks_by_label(&self, label: &str) -> Vec<&SignalTrack> {
        self.tracks.iter().filter(|t| t.label == label).collect()
    }

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

#[derive(Debug)]
pub enum SSTSBuilderError {
    Serialize {
        field: &'static str,
        source: serde_json::Error,
    },
}

impl Display for SSTSBuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize { field, source } => {
                write!(f, "failed to serialize {field}: {source}")
            }
        }
    }
}

impl Error for SSTSBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialize { source, .. } => Some(source),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SSTSBuilder {
    ssts: SSTS,
    track_by_label: HashMap<String, usize>,
}

impl Default for SSTSBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SSTSBuilder {
    pub fn new() -> Self {
        Self {
            ssts: SSTS::empty(),
            track_by_label: HashMap::new(),
        }
    }

    pub fn metadata_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.ssts.metadata
    }

    pub fn scalars_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.ssts.scalars
    }

    pub fn extra_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.ssts.extra
    }

    pub fn push_row_value(&mut self, label: impl Into<String>, time: Value, signal: Value) {
        let label = label.into();
        let idx = match self.track_by_label.get(&label) {
            Some(idx) => *idx,
            None => {
                let idx = self.ssts.tracks.len();
                self.ssts.tracks.push(SignalTrack::new(label.clone()));
                self.track_by_label.insert(label, idx);
                idx
            }
        };
        self.ssts.tracks[idx].push_row(time, signal);
    }

    pub fn push_row<T, U>(
        &mut self,
        label: impl Into<String>,
        time: T,
        signal: U,
    ) -> std::result::Result<(), SSTSBuilderError>
    where
        T: Serialize,
        U: Serialize,
    {
        let time_value = serde_json::to_value(time).map_err(|source| SSTSBuilderError::Serialize {
            field: "time",
            source,
        })?;
        let signal_value =
            serde_json::to_value(signal).map_err(|source| SSTSBuilderError::Serialize {
                field: "signal",
                source,
            })?;
        self.push_row_value(label, time_value, signal_value);
        Ok(())
    }

    pub fn build(self) -> SSTS {
        self.ssts
    }
}

#[derive(Debug)]
pub struct DecodeError {
    pub index: usize,
    pub source: serde_json::Error,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to decode value at index {}: {}", self.index, self.source)
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

pub fn decode_vec<T>(values: &[Value]) -> std::result::Result<Vec<T>, DecodeError>
where
    T: DeserializeOwned,
{
    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            serde_json::from_value::<T>(value.clone()).map_err(|source| DecodeError {
                index: idx,
                source,
            })
        })
        .collect()
}
