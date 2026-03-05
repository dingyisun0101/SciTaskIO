"""Checkpoint utilities for SSTS JSON epoch files."""

from pathlib import Path

from .io_json import load_ssts_json
from .validation import ValidationError


def _parse_checkpoint_epoch(path):
    """
    Parse checkpoint epoch from filename stem.

    Parameters:
    - `path`: candidate checkpoint file path.

    Returns:
    - Integer epoch when stem is numeric.
    - `None` when file is not a numeric checkpoint name.
    """
    file_path = Path(path)
    if file_path.suffix != ".json":
        return None
    stem = file_path.stem
    if not stem.isdigit():
        return None
    return int(stem)


def list_checkpoint_files(checkpoint_dir):
    """
    List checkpoint files in ascending epoch order.

    Parameters:
    - `checkpoint_dir`: directory containing `<epoch>.json` files.

    Returns:
    - List of `(epoch, path)` tuples sorted by epoch ascending.
    """
    root = Path(checkpoint_dir)
    items = []
    if not root.exists() or not root.is_dir():
        return items

    for path in root.iterdir():
        if not path.is_file():
            continue
        epoch = _parse_checkpoint_epoch(path)
        if epoch is None:
            continue
        items.append((epoch, path))

    items.sort(key=lambda item: item[0])
    return items


def latest_checkpoint_path(checkpoint_dir):
    """
    Resolve latest checkpoint file path.

    Parameters:
    - `checkpoint_dir`: directory containing epoch checkpoints.

    Returns:
    - `(epoch, path)` for highest epoch file, or `None` if none exists.
    """
    items = list_checkpoint_files(checkpoint_dir)
    if not items:
        return None
    return items[-1]


def preflight_checkpoint_dir(checkpoint_dir, require_ordering=False, remove_invalid=True):
    """
    Validate checkpoints and optionally remove invalid files.

    Parameters:
    - `checkpoint_dir`: directory containing epoch checkpoints.
    - `require_ordering`: passed through to SSTS validation.
    - `remove_invalid`: when True, delete invalid checkpoint files.

    Returns:
    - Summary dictionary with counts:
      - `scanned_files`
      - `valid_files`
      - `removed_invalid_files`
    """
    items = list_checkpoint_files(checkpoint_dir)
    scanned = len(items)
    valid = 0
    removed = 0

    for _, path in items:
        try:
            if path.stat().st_size == 0:
                raise ValidationError("empty checkpoint file")
            load_ssts_json(path, require_ordering=require_ordering)
            valid += 1
        except (ValidationError, OSError, ValueError):
            if remove_invalid and path.exists():
                path.unlink()
                removed += 1

    return {
        "scanned_files": scanned,
        "valid_files": valid,
        "removed_invalid_files": removed,
    }


def load_latest_checkpoint(checkpoint_dir, require_ordering=False, remove_invalid=True):
    """
    Load the latest valid checkpoint from a directory.

    Parameters:
    - `checkpoint_dir`: directory containing epoch checkpoints.
    - `require_ordering`: passed through to SSTS validation.
    - `remove_invalid`: when True, invalid files are deleted while searching.

    Returns:
    - `(epoch, ssts)` for the latest valid checkpoint.
    - `None` when no valid checkpoint exists.
    """
    items = list_checkpoint_files(checkpoint_dir)
    for epoch, path in reversed(items):
        try:
            if path.stat().st_size == 0:
                raise ValidationError("empty checkpoint file")
            ssts = load_ssts_json(path, require_ordering=require_ordering)
            return epoch, ssts
        except (ValidationError, OSError, ValueError):
            if remove_invalid and path.exists():
                path.unlink()
                continue
            raise

    return None


def prune_newer_than(checkpoint_dir, keep_epoch):
    """
    Remove checkpoint files newer than a target epoch.

    Parameters:
    - `checkpoint_dir`: directory containing epoch checkpoints.
    - `keep_epoch`: newest epoch to keep.

    Returns:
    - Number of removed files.
    """
    removed = 0
    for epoch, path in list_checkpoint_files(checkpoint_dir):
        if epoch <= int(keep_epoch):
            continue
        if path.exists():
            path.unlink()
            removed += 1
    return removed
