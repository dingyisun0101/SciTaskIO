"""Basic usage example for sci_task_io Python API."""

from pathlib import Path

from sci_task_io import (
    ModelCluster,
    SSTS,
    SSTSSeries,
    SignalTrack,
    ValidationError,
)


def main():
    root = Path(__file__).resolve().parents[2]
    fixture = root / "schema" / "fixtures" / "contract" / "valid" / "multi_track.json"

    # 1) Load an existing SSTS JSON file via OO alias.
    ssts = SSTS.from_json(fixture)
    print("loaded tracks:", [track.label for track in ssts.tracks])

    # 2) Build and save a new SSTS JSON file via OO alias.
    demo_ssts = SSTS(
        tracks=[
            SignalTrack(
                label="frequencies",
                times=[0, 10, 20],
                signal=[[0.5, 0.5], [0.6, 0.4], [0.55, 0.45]],
            ),
            SignalTrack(
                label="space",
                times=[0, 10],
                signal=[[[1, 0], [2, 1]], [[1, 2], [2, 0]]],
            ),
        ],
        metadata={"series_id": "demo_series", "source": "examples/basic_usage.py"},
        scalars={"seed": 123, "dt": 0.01},
    )

    out_root = root / "python" / "examples" / "output"
    out_json = out_root / "demo_series.json"
    demo_ssts.save_json(out_json)
    print("wrote:", out_json)

    # 3) Save one processed signal-label NPY file and load it back.
    combined_track = SignalTrack(
        label="frequencies",
        times=[0, 10, 20, 30],
        signal=[[0.5, 0.5], [0.6, 0.4], [0.55, 0.45], [0.52, 0.48]],
        extra={"sources": ["demo_series", "run_A"]},
    )

    out_npy = out_root / "frequencies.npy"
    combined_track.save_npy(out_npy)
    loaded_track = SignalTrack.from_npy(out_npy)
    print("loaded npy label:", loaded_track.label)
    print("loaded npy length:", len(loaded_track.times))

    # 4) Process one directory of SSTS JSON files into per-label NPY outputs.
    series_input = root / "schema" / "fixtures" / "contract" / "valid"
    series_out = out_root / "processed_from_series"
    ssts_series = SSTSSeries.from_dir(series_input)
    written = ssts_series.process(series_out)
    print("series wrote:", [path.name for path in written])

    # 5) Discover models under a root and filter by label entries.
    model_root = root / "DSES" / "output" / "PAF" / "persist_and_vary_energy" / "K-1000"
    model_cluster = ModelCluster(model_root)
    print("models discovered:", len(model_cluster.models))
    print("sample labels:", model_cluster.labels()[:3])

    # 6) Example validation error handling.
    try:
        SSTS.load_json(root / "schema" / "fixtures" / "contract" / "invalid" / "bad_track_key.json")
    except ValidationError as exc:
        print("caught expected validation error:", exc)


if __name__ == "__main__":
    main()
