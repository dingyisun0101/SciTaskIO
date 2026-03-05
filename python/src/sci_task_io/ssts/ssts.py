"""Core data models for SSTS payloads."""


class SignalTrack:
    """
    One labeled time-series track.

    Fields:
    - `label`: Track label (for example: "frequencies", "space").
    - `times`: Time index array for the track.
    - `signal`: Payload rows aligned 1:1 with `times`.
    - `extra`: Optional additional track-level fields to preserve.
    """

    def __init__(self, label, times=None, signal=None, extra=None):
        """
        Create a SignalTrack object.

        Parameters:
        - `label`: required non-empty track label.
        - `times`: optional list of time values; defaults to empty list.
        - `signal`: optional list of payload rows; defaults to empty list.
        - `extra`: optional dict of additional fields; defaults to empty dict.
        """
        self.label = label
        self.times = [] if times is None else list(times)
        self.signal = [] if signal is None else list(signal)
        self.extra = {} if extra is None else dict(extra)

    def to_payload(self):
        """
        Convert this track into canonical payload form.

        Returns:
        - Returns a dictionary with required keys `label`, `times`, `signal`
          plus any fields in `extra`.
        """
        payload = {
            "label": self.label,
            "times": self.times,
            "signal": self.signal,
        }
        payload.update(self.extra)
        return payload

    @classmethod
    def from_npy(cls, path, require_ordering=False):
        """
        OO alias for loading one processed signal-label `.npy` payload.

        Parameters:
        - `path`: filesystem path to `<signal_label>.npy`.
        - `require_ordering`: if True, enforce monotonic `times`.

        Returns:
        - Loaded `SignalTrack` object.
        """
        from .io_npy import load_signal_label_npy

        return load_signal_label_npy(path, require_ordering=require_ordering)

    @classmethod
    def load_npy(cls, path, require_ordering=False):
        """
        Alias of `from_npy` for method-style usage.
        """
        return cls.from_npy(path, require_ordering=require_ordering)

    def save_npy(self, path):
        """
        OO alias for saving this processed signal-label payload to `.npy`.

        Parameters:
        - `path`: destination filesystem path.
        """
        from .io_npy import save_signal_label_npy

        save_signal_label_npy(self, path)

    def to_npy(self, path):
        """
        Alias of `save_npy` for method-style usage.
        """
        self.save_npy(path)


class SSTS:
    """
    SSTS series payload for `<series_id>.json`.

    Fields:
    - `tracks`: Ordered list of SignalTrack instances; serialized as `track_1..track_N`.
    - `metadata`: Optional non-time-series metadata object.
    - `scalars`: Optional scalar key-value map.
    - `extra`: Optional additional top-level fields to preserve.
    """

    def __init__(self, tracks, metadata=None, scalars=None, extra=None):
        """
        Create an SSTS object.

        Parameters:
        - `tracks`: required ordered collection of SignalTrack objects.
        - `metadata`: optional object, defaults to empty dict.
        - `scalars`: optional scalar map, defaults to empty dict.
        - `extra`: optional map of preserved unknown top-level fields.
        """
        self.tracks = list(tracks)
        self.metadata = {} if metadata is None else dict(metadata)
        self.scalars = {} if scalars is None else dict(scalars)
        self.extra = {} if extra is None else dict(extra)

    def to_payload(self):
        """
        Convert this SSTS object into canonical JSON payload form.

        Returns:
        - Returns a top-level object with keys `metadata`, `scalars`, and `signals`.
        - `signals` is encoded as `track_1..track_N` from the current `tracks` order.
        """
        signals = {
            f"track_{idx}": track.to_payload() for idx, track in enumerate(self.tracks, start=1)
        }
        payload = {
            "metadata": self.metadata,
            "scalars": self.scalars,
            "signals": signals,
        }
        payload.update(self.extra)
        return payload

    @classmethod
    def from_json(cls, path, require_ordering=False):
        """
        OO alias for loading one SSTS JSON file.

        Parameters:
        - `path`: filesystem path to `<series_id>.json`.
        - `require_ordering`: if True, enforce monotonic `times`.

        Returns:
        - Loaded `SSTS` object.
        """
        from .io_json import load_ssts_json

        return load_ssts_json(path, require_ordering=require_ordering)

    @classmethod
    def load_json(cls, path, require_ordering=False):
        """
        Alias of `from_json` for method-style usage.
        """
        return cls.from_json(path, require_ordering=require_ordering)

    def save_json(self, path):
        """
        OO alias for saving this SSTS object to JSON.

        Parameters:
        - `path`: destination filesystem path.
        """
        from .io_json import save_ssts_json

        save_ssts_json(self, path)

    def to_json(self, path):
        """
        Alias of `save_json` for method-style usage.
        """
        self.save_json(path)
