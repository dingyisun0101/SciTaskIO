"""Standard label/path mapping rules for model discovery."""

from pathlib import Path


def _coerce_value(value):
    """
    Parse raw value text into a normalized scalar when possible.

    Behavior:
    - Converts integer-like strings to int.
    - Converts float-like strings to float.
    - Keeps non-numeric values as string.
    """
    text = str(value)
    try:
        if text.isdigit() or (text.startswith("-") and text[1:].isdigit()):
            return int(text)
        return float(text)
    except ValueError:
        return text


def _parse_token(token):
    """
    Parse one path token into one key-value pair.

    Behavior:
    - `key-value` becomes `{key: parsed(value)}`.
    - `key` becomes `{key: None}`.
    - Split uses the first `-` only.
    """
    text = str(token).strip()
    if not text:
        return None, None

    if "-" not in text:
        return text, None

    key, raw_value = text.split("-", 1)
    key = key.strip()
    if not key:
        return None, None
    return key, _coerce_value(raw_value.strip())


def path_to_label(root_path, model_path, strict=False):
    """
    Convert model path to a model label dictionary.

    Parameters:
    - `root_path`: cluster root path.
    - `model_path`: one discovered model directory path.
    - `strict`: if True, raise on duplicate keys.

    Behavior:
    - Uses relative path segments from root to model path.
    - In each segment, `_` separates tokens.
    - In each token, `-` separates key and value.
    - Plain tokens map to key with `None` value.
    """
    root = Path(root_path).resolve()
    model = Path(model_path).resolve()
    rel_parts = model.relative_to(root).parts

    label = {}
    for segment in rel_parts:
        for token in str(segment).split("_"):
            key, value = _parse_token(token)
            if key is None:
                continue
            if strict and key in label and label[key] != value:
                raise ValueError(f"Duplicate key with conflicting value: {key!r}")
            label[key] = value
    return label


def label_matches(label, filters):
    """
    Check whether one label satisfies filter key-value pairs.

    Behavior:
    - All filter pairs must match exactly.
    - Missing key is a mismatch.
    """
    for key, want in filters.items():
        if key not in label:
            return False
        if label[key] != want:
            return False
    return True
