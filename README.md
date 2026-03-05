# sci_task_io

`sci_task_io` is a cross-language library for standardized scientific time-series data.

It defines one canonical SSTS JSON contract and provides:
- a Python implementation for SSTS IO, processing, and model organization,
- a Rust implementation for SSTS JSON IO and validation.

## Project Scope

The project has three layers:

1. Schema contract
- Canonical SSTS format definition.
- Required top-level fields: `signals`; optional `metadata`, `scalars`.
- Tracks are ordered as `track_1`, `track_2`, ...
- Each track contains `label`, `times`, `signal`.

2. Python runtime (`python/`)
- Full SSTS API (JSON + processed NPY helpers).
- Series aggregation (`SSTSSeries`) and per-label NPY processing.
- Model organization layer (`Model`, `ModelCluster`) with automatic path-to-label mapping.
- Checkpoint utilities for epoch-style JSON files.

3. Rust runtime (`rust/`)
- SSTS JSON data model, validation, load/save.
- Directory-level SSTS series loading (`SSTSSeries`).
- Checkpoint utilities for epoch-style JSON files.
- No model layer and no NPY processing (by design, current scope).

## Repository Layout

- `schema/`
  - `schema.md`: human-readable SSTS contract.
  - `ssts.json.schema`: machine-readable JSON schema.
  - `signal_label.npy.schema.md`: processed NPY payload spec.
  - `fixtures/contract`: valid/invalid contract fixtures.

- `python/`
  - `src/sci_task_io/ssts`: SSTS core API.
  - `src/sci_task_io/model`: model/model-cluster abstractions.
  - `examples/`: runnable Python examples.

- `rust/`
  - `src/ssts`: SSTS core modules.
  - `examples/`: runnable Rust examples.

## SSTS Contract Summary

Canonical JSON file:
- `<series_id>.json`

Shape:
- `metadata` (optional object)
- `scalars` (optional scalar key-value object)
- `signals` (required object of `track_n` entries)

Track entry shape:
- `label`: non-empty string
- `times`: array of time values
- `signal`: array of rows, aligned 1:1 with `times`

Processed NPY (Python-oriented):
- `<signal_label>.npy`
- stores one combined time series for one label.

## Python API Overview

Main exports:
- `SSTS`, `SignalTrack`, `SSTSSeries`
- `Model`, `ModelCluster`
- validation and IO helpers
- checkpoint utilities

OO-style examples:
- `SSTS.from_json(path)` / `ssts.save_json(path)`
- `SignalTrack.from_npy(path)` / `track.save_npy(path)`
- `SSTSSeries.from_dir(path).process(out_dir)`

Model layer:
- a `Model` = one directory containing valid SSTS JSON files.
- a `ModelCluster` = discovered set of models under a root, with label-based filtering.

## Rust API Overview

Main module:
- `sci_task_io::ssts`

Includes:
- `SSTS`, `SignalTrack`
- JSON IO and validation
- `SSTSSeries` directory loader
- checkpoint utilities

OO-style examples:
- `SSTS::from_json(path, require_ordering)`
- `ssts.save_json(path)`
- `SSTSSeries::from_dir(path, require_ordering)`

Current boundary:
- Rust does not implement model discovery or NPY processing yet.

## Examples

Python:
- `python/examples/basic_usage.py`

Rust:
- `rust/examples/basic_usage.rs`
- `rust/examples/series_usage.rs`

## Notes

- Python is currently validated against `/home/mgr/micromamba/envs/phys`.
- Rust compilation in restricted environments may require network access for Cargo dependencies.
