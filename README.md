# sci_task_io (Python)

`sci_task_io` standardizes time-series scientific data storage and processing so Rust and Python can use the same structures consistently.

## What SSTS Is

SSTS (System State Time Series) is the core format for one calculation result series.

Each SSTS JSON file (`<series_id>.json`) contains:
- `metadata`: descriptive metadata for the series,
- `scalars`: scalar key-value attributes,
- `signals`: ordered tracks (`track_1`, `track_2`, ...) where each track has:
  - `label`,
  - `times`,
  - `signal`.

SSTS is intentionally generic:
- every track is a time series,
- semantic meaning comes from `label` (for example `frequencies`, `space`, `energy`),
- row payload shape is not hardcoded by the core IO layer.

## Processed NPY Outputs

For fast downstream Python I/O, SSTS series can be processed into per-label NPY files:
- `<signal_label>.npy`

Each processed file stores one combined time series for one label, merged from multiple source SSTS JSON files.

## Why Model Exists

SSTS solves data format consistency. `Model` and `ModelCluster` solve data organization.

Scientific runs are usually stored as many directories with parameterized path names. The model layer provides a standard way to map those paths into structured labels and query them.

## Model Layer

### `Model`

A `Model` is one directory containing valid SSTS JSON files.

`Model(path)` automatically creates an `SSTSSeries` for that directory so you can:
- inspect serial ids,
- process series to per-label NPY outputs.

### `ModelCluster`

A `ModelCluster` indexes many models under a root directory.

`ModelCluster(root_path)`:
- scans for valid model directories,
- maps each directory path to a label dictionary,
- supports filtering and selection via key-value pairs,
- keeps discovery/indexing lightweight (does not keep all series payloads in RAM).

## Label/Path Mapping Rule

Path tokens are mapped to label entries with this standard:
- `_` separates multiple entries in one folder segment,
- `-` separates key and value in one token,
- plain token without `-` maps to `{token: None}`.

Examples:
- `SDE` -> `{"SDE": None}`
- `n-0.00000` -> `{"n": 0.0}`
- `A-1_B-2` -> `{"A": 1, "B": 2}`

## Minimal Usage

```python
from pathlib import Path
from sci_task_io import ModelCluster

root = Path("/path/to/output/K-1000")
cluster = ModelCluster(root)

# list discovered model labels
print(cluster.labels())

# filter models by mapped label entries
matches = cluster.filter(SDE=None, n=0.0)

# process one model to per-label npy outputs
if matches:
    model = matches[0]
    model.process("/tmp/processed")
```

## Main Python APIs

SSTS APIs:
- `SSTS`, `SignalTrack`, `SSTSSeries`
- `load_ssts_json`, `save_ssts_json`
- `load_signal_label_npy`, `save_signal_label_npy`
- `validate_ssts_payload`, `ValidationError`

Model APIs:
- `Model`
- `ModelCluster`
- `path_to_label`, `label_matches`
