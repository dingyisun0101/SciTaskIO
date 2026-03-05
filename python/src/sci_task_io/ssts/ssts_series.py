"""Cluster utilities for bulk SSTS JSON processing."""

from pathlib import Path

from .io_json import load_ssts_json
from .io_npy import save_signal_label_npy
from .ssts import SignalTrack


class SSTSSeries:
    """
    Collection of SSTS series files under one directory.

    Fields:
    - `root_dir`: Directory containing many `<series_id>.json` files.
    - `require_ordering`: If True, enforce monotonic `times` while loading.
    - `entries`: Loaded entries sorted by serial id.

    Notes:
    - Serial id is resolved from `metadata.serial_id`, then `metadata.series_id`,
      then JSON filename stem.
    """

    def __init__(self, root_dir, require_ordering=False):
        """
        Create an SSTSSeries instance.

        Parameters:
        - `root_dir`: Directory path containing SSTS JSON files.
        - `require_ordering`: If True, enforce ordering validation at load time.
        """
        self.root_dir = Path(root_dir)
        self.require_ordering = require_ordering
        self.entries = []

    def _iter_series_paths(self):
        """
        Yield candidate SSTS JSON paths from the cluster root directory.

        Behavior:
        - Non-recursive scan of `*.json`.
        - Excludes `*.scalars.json` sidecar files.
        """
        for path in sorted(self.root_dir.glob("*.json"), key=lambda p: p.name):
            if path.name.endswith(".scalars.json"):
                continue
            if not path.is_file():
                continue
            yield path

    def _resolve_serial_id(self, ssts, path):
        """
        Resolve serial id used for cluster sorting.

        Behavior:
        - Prefer `metadata.serial_id`.
        - Fallback to `metadata.series_id`.
        - Fallback to filename stem.
        """
        metadata = ssts.metadata if isinstance(ssts.metadata, dict) else {}
        if "serial_id" in metadata and metadata["serial_id"] is not None:
            return str(metadata["serial_id"])
        if "series_id" in metadata and metadata["series_id"] is not None:
            return str(metadata["series_id"])
        return path.stem

    def load(self):
        """
        Load all SSTS JSON files from the cluster directory.

        Returns:
        - `self`, with `entries` populated and sorted by serial id.
        """
        entries = []
        for path in self._iter_series_paths():
            ssts = load_ssts_json(path, require_ordering=self.require_ordering)
            serial_id = self._resolve_serial_id(ssts, path)
            entries.append({
                "serial_id": serial_id,
                "path": path,
                "ssts": ssts,
            })

        entries.sort(key=lambda item: item["serial_id"])
        self.entries = entries
        return self

    def serial_ids(self):
        """
        Return serial ids in current cluster order.

        Behavior:
        - If not loaded yet, this method loads first.
        """
        if not self.entries:
            self.load()
        return [item["serial_id"] for item in self.entries]

    def _time_sort_key(self, value):
        """
        Create deterministic ordering key for time values.

        Behavior:
        - Numbers sort before strings.
        - Numbers sort by numeric value.
        - Strings sort lexicographically.
        """
        if isinstance(value, (int, float)):
            return (0, float(value))
        return (1, str(value))

    def process(self, output_dir):
        """
        Build processed `<signal_label>.npy` files from all cluster series.

        Parameters:
        - `output_dir`: Directory where processed `.npy` outputs are written.

        Returns:
        - List of written `.npy` paths.

        Behavior:
        - Combines all rows from all series for each track label.
        - Sorts combined rows by time with deterministic tie-breakers.
        - Saves one output file per label: `<label>.npy`.
        """
        if not self.entries:
            self.load()

        output_root = Path(output_dir)
        output_root.mkdir(parents=True, exist_ok=True)

        by_label = {}

        for source_idx, item in enumerate(self.entries):
            serial_id = item["serial_id"]
            ssts = item["ssts"]

            for track in ssts.tracks:
                label = track.label
                if label not in by_label:
                    by_label[label] = []

                for row_idx, (time_value, row_value) in enumerate(zip(track.times, track.signal)):
                    by_label[label].append(
                        (time_value, row_value, serial_id, source_idx, row_idx)
                    )

        written = []
        for label, rows in by_label.items():
            rows.sort(key=lambda item: (
                self._time_sort_key(item[0]),
                item[2],
                item[3],
                item[4],
            ))

            combined_times = [item[0] for item in rows]
            combined_signal = [item[1] for item in rows]
            sources = sorted({item[2] for item in rows})

            track = SignalTrack(
                label=label,
                times=combined_times,
                signal=combined_signal,
                extra={
                    "sources": sources,
                    "source_count": len(sources),
                },
            )

            path = output_root / f"{label}.npy"
            save_signal_label_npy(track, path)
            written.append(path)

        return written
