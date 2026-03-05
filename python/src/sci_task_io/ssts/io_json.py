"""JSON IO for SSTS `<series_id>.json` payloads."""

import json
from pathlib import Path

from .ssts import SSTS
from .validation import ssts_from_payload


def load_ssts_json(path, require_ordering=False):
    """
    Load and validate one SSTS JSON file.

    Parameters:
    - `path`: filesystem path to `<series_id>.json`.
    - `require_ordering`: if True, enforces monotonic `times` in each track.
    - Returns an in-memory `SSTS` object.
    """
    file_path = Path(path)
    payload = json.loads(file_path.read_text())
    return ssts_from_payload(payload, require_ordering=require_ordering)


def save_ssts_json(ssts, path):
    """
    Save one SSTS JSON file in canonical shape.

    Parameters:
    - `ssts`: in-memory SSTS object.
    - `path`: destination filesystem path.
    - Writes pretty-printed JSON with canonical top-level keys.
    """
    if not isinstance(ssts, SSTS):
        raise TypeError("save_ssts_json expects an SSTS instance.")

    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    file_path.write_text(json.dumps(ssts.to_payload(), indent=2))
