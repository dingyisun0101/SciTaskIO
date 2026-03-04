# SSTS Schema

## 1. Purpose

This document defines the canonical persisted schema for SSTS (System State Time Series) used by `sci_task_io`.

The primary goal is to **standardize time-series scientific data** into one consistent, language-neutral type and storage contract.

This standard lets Rust and Python read and write the same scientific time-series structures consistently.

## 2. Canonical File Layout

Processed SSTS data is stored in a single JSON file:
- `<series_id>.json`

Processed NumPy output stores a single signal track aggregated from multiple JSON series files:
- `<signal_label>.npy` (for example `frequencies.npy`)

The largest top-level fields are:
- `metadata`
- `scalars`
- `signals`

## 3. Top-Level Object

An SSTS JSON object contains:
- `metadata`: optional object for non-time-series metadata.
- `scalars`: optional object for scalar key-value pairs (not time series).
- `signals`: required object that stores ordered signal tracks.

Recommended shape:

```json
{
  "metadata": {},
  "scalars": {},
  "signals": {}
}
```

## 4. Signals Container

`signals` stores an arbitrary number of tracks in order using explicit keys:
- `track_1`
- `track_2`
- `track_3`
- ...

Ordering is defined by numeric suffix (`track_1`, `track_2`, ...), not by JSON object insertion order.

Every `track_n` is a time series by definition.

## 5. Track Schema

Each `track_n` object must contain:
- `label`: string label for the track, for example `"space"` or `"frequencies"`.
- `times`: list of time indices.
- `signal`: list of payload rows aligned 1:1 with `times`.

`times` and `signal` together define one time series.

Track object example:

```json
{
  "label": "frequencies",
  "times": [0, 10, 20],
  "signal": [
    [0.5, 0.5],
    [0.6, 0.4],
    [0.55, 0.45]
  ]
}
```

## 6. Field Semantics

### 6.1 `metadata`

- Optional.
- Arbitrary object for descriptive metadata.
- May contain nested structures.

### 6.2 `scalars`

- Optional.
- Arbitrary key-value map for scalar attributes.
- Values should be scalar JSON types (`number`, `string`, `boolean`, `null`).

### 6.3 `signals`

- Required.
- Object containing `track_1`, `track_2`, ...
- Every track follows the track schema in section 5.
- Every track must represent a time series (`times` + aligned `signal` rows).

## 7. Validation Rules

### 7.1 Container-level

- `signals` must be an object.
- Track keys must match `track_<positive_integer>`.
- Track numbering should be contiguous starting at 1 (`track_1..track_N`).

### 7.2 Track-level

- `label` must be a non-empty string.
- `times` and `signal` must both be arrays.
- `len(times) == len(signal)`.
- `times` should be monotonic non-decreasing when ordering is required.

### 7.3 Strict vs permissive reads

Strict mode:
- reject malformed track keys,
- reject missing required track fields,
- reject mismatched `times`/`signal` lengths.

Permissive mode:
- skip malformed tracks when possible,
- continue loading remaining valid tracks,
- surface warnings.

## 8. Compatibility Policy

Allowed without breaking schema:
- adding optional top-level fields,
- adding optional track-level fields,
- adding reader options that do not change default semantics.

Requires a breaking schema revision:
- removing required fields,
- renaming required fields,
- changing meaning of existing required fields.

## 9. Processed NumPy Form

The NumPy processed form is **track-centric**, not series-centric.

One `.npy` file stores one signal label merged from multiple JSON series inputs.

Naming convention:
- `<signal_label>.npy` (for example `frequencies.npy`, `space.npy`)

Recommended content convention:
- dict-like payload with at least:
  - `label`: the signal label (matches filename stem),
  - `times`: one combined time axis,
  - `signal`: one combined signal payload aligned with `times`.

Processing intent:
- collect all time steps for one signal label from all source series,
- combine them into one large time series,
- store that large time series in `<signal_label>.npy` for fast downstream Python I/O.

Recommended ordering rule for combined data:
- sort combined rows by time with deterministic tiebreakers when times are equal.

Recommended write/read convention:
- write one dict-like object to `<signal_label>.npy`
- use pickle-enabled object array support for nested structures when needed
- reading should reconstruct one combined time series for that label

## 10. Canonical Example

```json
{
  "metadata": {
    "series_id": "run_A",
    "source": "simulation",
    "notes": "pilot"
  },
  "scalars": {
    "seed": 42,
    "dt": 0.01
  },
  "signals": {
    "track_1": {
      "label": "frequencies",
      "times": [0, 10, 20],
      "signal": [
        [0.5, 0.5],
        [0.6, 0.4],
        [0.55, 0.45]
      ]
    },
    "track_2": {
      "label": "space",
      "times": [0, 10],
      "signal": [
        [[1, 0], [2, 1]],
        [[1, 2], [2, 0]]
      ]
    }
  }
}
```

## 11. Notes for Implementers

- Treat track labels as semantic hints for downstream tools.
- Do not hardcode special behavior by label in core IO.
- Keep core parsing generic and deterministic.
