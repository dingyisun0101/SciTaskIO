"""SSTS core IO and series processing APIs."""

from .io_json import load_ssts_json, save_ssts_json
from .io_npy import load_signal_label_npy, save_signal_label_npy
from .ssts import SSTS, SignalTrack
from .ssts_series import SSTSSeries
from .validation import ValidationError, ssts_from_payload, validate_ssts_payload

__all__ = [
    "SSTS",
    "SignalTrack",
    "SSTSSeries",
    "ValidationError",
    "validate_ssts_payload",
    "ssts_from_payload",
    "load_ssts_json",
    "save_ssts_json",
    "load_signal_label_npy",
    "save_signal_label_npy",
]
