use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use serde_json::{Map, Value};

use super::ssts::{SSTS, SignalTrack};

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ValidationError {}

fn compare_time_values(left: &Value, right: &Value) -> Result<Ordering, ValidationError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a
                .as_f64()
                .ok_or_else(|| ValidationError::new("invalid numeric time value"))?;
            let b = b
                .as_f64()
                .ok_or_else(|| ValidationError::new("invalid numeric time value"))?;
            Ok(a.partial_cmp(&b).unwrap_or(Ordering::Equal))
        }
        (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
        (Value::Number(_), Value::String(_)) => Ok(Ordering::Less),
        (Value::String(_), Value::Number(_)) => Ok(Ordering::Greater),
        _ => Err(ValidationError::new("time values must be numbers or strings")),
    }
}

fn validate_time_values(times: &[Value], require_ordering: bool) -> Result<(), ValidationError> {
    for value in times {
        if !value.is_number() && !value.is_string() {
            return Err(ValidationError::new("time values must be numbers or strings"));
        }
    }

    if require_ordering {
        for pair in times.windows(2) {
            if compare_time_values(&pair[0], &pair[1])? == Ordering::Greater {
                return Err(ValidationError::new(
                    "times must be monotonic non-decreasing",
                ));
            }
        }
    }

    Ok(())
}

fn parse_track_key(key: &str) -> Option<usize> {
    let suffix = key.strip_prefix("track_")?;
    if suffix.is_empty() {
        return None;
    }
    let n = suffix.parse::<usize>().ok()?;
    if n == 0 {
        return None;
    }
    Some(n)
}

fn validate_signal_track_payload(
    payload: &Map<String, Value>,
    require_ordering: bool,
) -> Result<(), ValidationError> {
    let label = payload
        .get("label")
        .ok_or_else(|| ValidationError::new("track payload must contain label"))?;
    let times = payload
        .get("times")
        .ok_or_else(|| ValidationError::new("track payload must contain times"))?;
    let signal = payload
        .get("signal")
        .ok_or_else(|| ValidationError::new("track payload must contain signal"))?;

    let label = label
        .as_str()
        .ok_or_else(|| ValidationError::new("track label must be a non-empty string"))?;
    if label.is_empty() {
        return Err(ValidationError::new("track label must be a non-empty string"));
    }

    let times = times
        .as_array()
        .ok_or_else(|| ValidationError::new("track times must be an array"))?;
    let signal = signal
        .as_array()
        .ok_or_else(|| ValidationError::new("track signal must be an array"))?;

    if times.len() != signal.len() {
        return Err(ValidationError::new(
            "track times and signal lengths must match",
        ));
    }

    validate_time_values(times, require_ordering)
}

fn validate_scalars(scalars: &Map<String, Value>) -> Result<(), ValidationError> {
    for (k, v) in scalars {
        if k.is_empty() {
            return Err(ValidationError::new("scalars keys must be non-empty strings"));
        }
        if !(v.is_number() || v.is_string() || v.is_boolean() || v.is_null()) {
            return Err(ValidationError::new(
                "scalars values must be scalar JSON values",
            ));
        }
    }
    Ok(())
}

pub fn validate_ssts_payload(payload: &Value, require_ordering: bool) -> Result<(), ValidationError> {
    let obj = payload
        .as_object()
        .ok_or_else(|| ValidationError::new("SSTS payload must be an object"))?;

    if let Some(metadata) = obj.get("metadata") {
        if !metadata.is_object() {
            return Err(ValidationError::new("metadata must be an object when provided"));
        }
    }

    if let Some(scalars) = obj.get("scalars") {
        let scalars = scalars
            .as_object()
            .ok_or_else(|| ValidationError::new("scalars must be an object when provided"))?;
        validate_scalars(scalars)?;
    }

    let signals = obj
        .get("signals")
        .ok_or_else(|| ValidationError::new("SSTS payload must contain signals"))?
        .as_object()
        .ok_or_else(|| ValidationError::new("signals must be an object"))?;

    let mut ids = BTreeSet::new();
    for (key, track) in signals {
        let id = parse_track_key(key)
            .ok_or_else(|| ValidationError::new(format!("invalid track key: {key:?}")))?;
        ids.insert(id);

        let track = track
            .as_object()
            .ok_or_else(|| ValidationError::new("track payload must be an object"))?;
        validate_signal_track_payload(track, require_ordering)?;
    }

    if let Some(max_id) = ids.iter().max().copied() {
        for expected in 1..=max_id {
            if !ids.contains(&expected) {
                return Err(ValidationError::new(
                    "track keys must be contiguous from track_1 to track_N",
                ));
            }
        }
    }

    Ok(())
}

pub fn ssts_from_payload(payload: &Value, require_ordering: bool) -> Result<SSTS, ValidationError> {
    validate_ssts_payload(payload, require_ordering)?;

    let obj = payload
        .as_object()
        .ok_or_else(|| ValidationError::new("SSTS payload must be an object"))?;

    let metadata = obj
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let scalars = obj
        .get("scalars")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let known = ["metadata", "scalars", "signals"];
    let mut extra = Map::new();
    for (k, v) in obj {
        if !known.contains(&k.as_str()) {
            extra.insert(k.clone(), v.clone());
        }
    }

    let signals = obj
        .get("signals")
        .and_then(Value::as_object)
        .ok_or_else(|| ValidationError::new("signals must be an object"))?;

    let mut tracks = Vec::new();
    for idx in 1..=signals.len() {
        let key = format!("track_{idx}");
        let track_obj = signals
            .get(&key)
            .and_then(Value::as_object)
            .ok_or_else(|| ValidationError::new(format!("missing contiguous track: {key}")))?;

        let label = track_obj
            .get("label")
            .and_then(Value::as_str)
            .ok_or_else(|| ValidationError::new("track label must be a non-empty string"))?
            .to_string();
        let times = track_obj
            .get("times")
            .and_then(Value::as_array)
            .ok_or_else(|| ValidationError::new("track times must be an array"))?
            .clone();
        let signal = track_obj
            .get("signal")
            .and_then(Value::as_array)
            .ok_or_else(|| ValidationError::new("track signal must be an array"))?
            .clone();

        let mut track_extra = Map::new();
        for (k, v) in track_obj {
            if k != "label" && k != "times" && k != "signal" {
                track_extra.insert(k.clone(), v.clone());
            }
        }

        tracks.push(SignalTrack {
            label,
            times,
            signal,
            extra: track_extra,
        });
    }

    Ok(SSTS {
        tracks,
        metadata,
        scalars,
        extra,
    })
}
