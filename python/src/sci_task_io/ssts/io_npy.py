"""NumPy IO for processed `<signal_label>.npy` payloads."""

from pathlib import Path

from .ssts import SignalTrack
from .validation import validate_signal_label_npy_payload


def load_signal_label_npy(path, require_ordering=False):
    """
    Load one processed signal-label `.npy` payload.

    Parameters:
    - `path`: filesystem path to `<signal_label>.npy`.
    - `require_ordering`: if True, enforces monotonic `times`.
    - Returns a `SignalTrack` object.
    """
    import numpy as np

    file_path = Path(path)
    with file_path.open("rb") as f:
        payload = np.load(f, allow_pickle=True)
        payload = payload.item() if hasattr(payload, "item") else payload

    if not isinstance(payload, dict):
        raise ValueError("npy payload must decode to a dict-like object.")

    validate_signal_label_npy_payload(
        payload,
        filename_stem=file_path.stem,
        require_ordering=require_ordering,
    )

    extra = {k: v for k, v in payload.items() if k not in {"label", "times", "signal"}}
    return SignalTrack(
        label=payload["label"],
        times=list(payload["times"]),
        signal=list(payload["signal"]),
        extra=extra,
    )


def save_signal_label_npy(track, path):
    """
    Save one processed signal-label `.npy` payload.

    Parameters:
    - `track`: SignalTrack instance.
    - `path`: destination path whose filename stem must match `track.label`.
    - Persists a single dict-like payload using NumPy with pickle enabled.
    """
    import numpy as np

    if not isinstance(track, SignalTrack):
        raise TypeError("save_signal_label_npy expects a SignalTrack instance.")

    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)

    payload = track.to_payload()
    validate_signal_label_npy_payload(payload, filename_stem=file_path.stem)

    with file_path.open("wb") as f:
        np.save(f, payload, allow_pickle=True)
