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
