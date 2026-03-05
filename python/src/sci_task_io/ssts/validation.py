"""Validation helpers for SSTS and processed signal NPY payloads."""

import re

from .ssts import SSTS, SignalTrack

_TRACK_KEY_RE = re.compile(r"^track_([1-9][0-9]*)$")


class ValidationError(ValueError):
    """
    Raised when payload validation fails.

    Notes:
    - This is the canonical validation exception for schema and runtime checks.
    """


def _is_scalar(value):
    """
    Check whether a value is a scalar JSON-compatible value.

    Behavior:
    - Accepts: number, string, boolean, null.
    - Returns True when accepted by `scalars` field contract.
    """
    return value is None or isinstance(value, (bool, int, float, str))


def _is_time_value(value):
    """
    Check whether a value is a valid time value.

    Behavior:
    - Accepts: number or string.
    - Used to validate `times` arrays.
    """
    return isinstance(value, (int, float, str))


def _validate_monotonic(times, require_ordering):
    """
    Validate monotonic non-decreasing time ordering when requested.

    Parameters:
    - `times`: sequence of time values.
    - `require_ordering`: if False, ordering checks are skipped.
    - Raises ValidationError on decreasing or non-comparable adjacent values.
    """
    if not require_ordering or len(times) < 2:
        return
    for left, right in zip(times[:-1], times[1:]):
        try:
            if left > right:
                raise ValidationError("times must be monotonic non-decreasing.")
        except TypeError as exc:
            raise ValidationError("times contains non-comparable values.") from exc


def validate_signal_track_payload(payload, require_ordering=False):
    """
    Validate one `track_n` payload.

    Behavior:
    - Requires keys: `label`, `times`, `signal`.
    - Enforces non-empty string label.
    - Enforces `times` and `signal` arrays with equal length.
    - Optionally enforces monotonic ordering of `times`.
    """
    if not isinstance(payload, dict):
        raise ValidationError("track payload must be an object.")

    if "label" not in payload or "times" not in payload or "signal" not in payload:
        raise ValidationError("track payload must contain label, times, and signal.")

    label = payload["label"]
    times = payload["times"]
    signal = payload["signal"]

    if not isinstance(label, str) or not label:
        raise ValidationError("track label must be a non-empty string.")
    if not isinstance(times, list):
        raise ValidationError("track times must be an array.")
    if not isinstance(signal, list):
        raise ValidationError("track signal must be an array.")
    if len(times) != len(signal):
        raise ValidationError("track times and signal lengths must match.")

    for value in times:
        if not _is_time_value(value):
            raise ValidationError("time values must be numbers or strings.")

    _validate_monotonic(times, require_ordering=require_ordering)


def validate_ssts_payload(payload, require_ordering=False):
    """
    Validate full SSTS payload for `<series_id>.json`.

    Behavior:
    - Requires top-level `signals` object.
    - Validates optional `metadata` and `scalars` object types.
    - Enforces `signals` keys match `track_<positive_integer>`.
    - Enforces contiguous track numbering `track_1..track_N`.
    - Validates each track via `validate_signal_track_payload`.
    """
    if not isinstance(payload, dict):
        raise ValidationError("SSTS payload must be an object.")

    if "signals" not in payload:
        raise ValidationError("SSTS payload must contain signals.")

    metadata = payload.get("metadata", {})
    scalars = payload.get("scalars", {})
    signals = payload["signals"]

    if metadata is not None and not isinstance(metadata, dict):
        raise ValidationError("metadata must be an object when provided.")
    if scalars is not None and not isinstance(scalars, dict):
        raise ValidationError("scalars must be an object when provided.")

    if isinstance(scalars, dict):
        for key, value in scalars.items():
            if not isinstance(key, str):
                raise ValidationError("scalars keys must be strings.")
            if not _is_scalar(value):
                raise ValidationError("scalars values must be scalar JSON values.")

    if not isinstance(signals, dict):
        raise ValidationError("signals must be an object.")

    track_numbers = []
    for key, track_payload in signals.items():
        match = _TRACK_KEY_RE.match(key)
        if not match:
            raise ValidationError(f"invalid track key: {key!r}.")
        track_numbers.append(int(match.group(1)))
        validate_signal_track_payload(track_payload, require_ordering=require_ordering)

    if track_numbers:
        expected = list(range(1, max(track_numbers) + 1))
        if sorted(track_numbers) != expected:
            raise ValidationError("track keys must be contiguous from track_1 to track_N.")


def ssts_from_payload(payload, require_ordering=False):
    """
    Build an `SSTS` object from raw payload after validation.

    Behavior:
    - Validates payload first.
    - Preserves unknown top-level and track-level fields in `.extra`.
    - Returns normalized in-memory `SSTS` and `SignalTrack` objects.
    """
    validate_ssts_payload(payload, require_ordering=require_ordering)

    known = {"metadata", "scalars", "signals"}
    extra = {k: v for k, v in payload.items() if k not in known}

    signals = payload["signals"]
    tracks = []
    for idx in range(1, len(signals) + 1):
        track_obj = signals[f"track_{idx}"]
        track_extra = {k: v for k, v in track_obj.items() if k not in {"label", "times", "signal"}}
        tracks.append(
            SignalTrack(
                label=track_obj["label"],
                times=list(track_obj["times"]),
                signal=list(track_obj["signal"]),
                extra=track_extra,
            )
        )

    metadata = payload.get("metadata") or {}
    scalars = payload.get("scalars") or {}
    return SSTS(tracks=tracks, metadata=dict(metadata), scalars=dict(scalars), extra=extra)


def validate_signal_label_npy_payload(payload, filename_stem=None, require_ordering=False):
    """
    Validate one processed `<signal_label>.npy` payload.

    Behavior:
    - Applies standard track validation (`label`, `times`, `signal`).
    - If `filename_stem` is provided, enforces `payload['label'] == filename_stem`.
    """
    validate_signal_track_payload(payload, require_ordering=require_ordering)
    if filename_stem is not None and payload["label"] != filename_stem:
        raise ValidationError("npy payload label must match filename stem.")
