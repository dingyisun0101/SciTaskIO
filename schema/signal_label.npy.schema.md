# `<signal_label>.npy` Payload Schema

This document defines the logical schema stored inside one processed NumPy file named `<signal_label>.npy`.

## Purpose

`<signal_label>.npy` stores one combined time series for exactly one signal label, merged from multiple `<series_id>.json` files.

## Filename Contract

- Filename must be `<signal_label>.npy`.
- Payload `label` must equal the filename stem `<signal_label>`.

## Logical Payload Shape

The `.npy` payload is a dict-like object with required keys:

- `label`: string
- `times`: array
- `signal`: array

Minimal payload example:

```python
{
  "label": "frequencies",
  "times": [0, 10, 20, 30],
  "signal": [
    [0.5, 0.5],
    [0.6, 0.4],
    [0.55, 0.45],
    [0.52, 0.48],
  ],
}
```

## Field Rules

- `label`
  - non-empty string
  - must match filename stem

- `times`
  - array of time values (`number` or `string`)

- `signal`
  - array of payload rows
  - row type is intentionally generic

## Required Runtime Validation

The following must be enforced by loader/writer code:

1. `len(times) == len(signal)`
2. combined rows come from all source series for this label
3. output is sorted by time when ordering is required
4. equal-time rows use deterministic tie-breakers

## Optional Provenance Fields

Optional fields are allowed, for example:

- `sources`: list of source series ids or paths
- `source_count`: integer
- `generated_at`: timestamp string
- `metadata`: object

Readers should ignore unknown fields.
