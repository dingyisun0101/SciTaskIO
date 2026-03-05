use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Result;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct SignalTrack {
    pub label: String,
    pub times: Vec<Value>,
    pub signal: Vec<Value>,
    pub extra: Map<String, Value>,
}

impl Serialize for SignalTrack {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3 + self.extra.len()))?;
        map.serialize_entry("label", &self.label)?;
        map.serialize_entry("times", &self.times)?;
        map.serialize_entry("signal", &self.signal)?;
        for (k, v) in &self.extra {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
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

    pub fn with_capacity(
        label: impl Into<String>,
        times_capacity: usize,
        signal_capacity: usize,
    ) -> Self {
        Self {
            label: label.into(),
            times: Vec::with_capacity(times_capacity),
            signal: Vec::with_capacity(signal_capacity),
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

struct SignalsByIndex<'a>(&'a [SignalTrack]);

impl Serialize for SignalsByIndex<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (idx, track) in self.0.iter().enumerate() {
            map.serialize_entry(&format!("track_{}", idx + 1), track)?;
        }
        map.end()
    }
}

impl Serialize for SSTS {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3 + self.extra.len()))?;
        map.serialize_entry("metadata", &self.metadata)?;
        map.serialize_entry("scalars", &self.scalars)?;
        map.serialize_entry("signals", &SignalsByIndex(&self.tracks))?;
        for (k, v) in &self.extra {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
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
        serde_json::to_value(self).expect("SSTS serialization to Value should not fail")
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

    pub fn with_track_capacity(track_capacity: usize) -> Self {
        Self {
            ssts: SSTS {
                tracks: Vec::with_capacity(track_capacity),
                metadata: Map::new(),
                scalars: Map::new(),
                extra: Map::new(),
            },
            track_by_label: HashMap::with_capacity(track_capacity),
        }
    }

    pub fn reserve_tracks(&mut self, additional: usize) {
        self.ssts.tracks.reserve(additional);
        self.track_by_label.reserve(additional);
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

    fn find_track_index(&self, label: &str) -> Option<usize> {
        self.track_by_label.get(label).copied()
    }

    fn ensure_track_index(&mut self, label: String) -> usize {
        if let Some(idx) = self.track_by_label.get(&label).copied() {
            return idx;
        }
        let idx = self.ssts.tracks.len();
        self.ssts.tracks.push(SignalTrack::new(label.clone()));
        self.track_by_label.insert(label, idx);
        idx
    }

    pub fn reserve_rows_for_label(&mut self, label: &str, additional: usize) {
        if let Some(idx) = self.find_track_index(label) {
            self.ssts.tracks[idx].times.reserve(additional);
            self.ssts.tracks[idx].signal.reserve(additional);
        }
    }

    pub fn push_row_value_ref(&mut self, label: &str, time: Value, signal: Value) {
        let idx = match self.find_track_index(label) {
            Some(idx) => idx,
            None => self.ensure_track_index(label.to_string()),
        };
        self.ssts.tracks[idx].push_row(time, signal);
    }

    pub fn push_row_value(&mut self, label: impl Into<String>, time: Value, signal: Value) {
        let label = label.into();
        let idx = self.ensure_track_index(label);
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
        let time_value =
            serde_json::to_value(time).map_err(|source| SSTSBuilderError::Serialize {
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
        write!(
            f,
            "failed to decode value at index {}: {}",
            self.index, self.source
        )
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
            T::deserialize(value).map_err(|source| DecodeError { index: idx, source })
        })
        .collect()
}
