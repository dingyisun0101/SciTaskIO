"""Model abstraction built on top of SSTS series directories."""

from pathlib import Path

from ..ssts import SSTSSeries


class Model:
    """
    A model is one directory containing valid SSTS JSON files.

    Fields:
    - `path`: filesystem directory path for the model.
    - `label`: dictionary label parsed from path mapping rules.
    - `ssts_series`: lazy SSTS series handler for files in this model directory.
    """

    def __init__(self, path, label=None, require_ordering=False):
        """
        Create a Model instance.

        Parameters:
        - `path`: model directory path.
        - `label`: optional precomputed label dictionary.
        - `require_ordering`: passes through to SSTS series loading behavior.
        """
        self.path = Path(path)
        self.label = {} if label is None else dict(label)
        self.ssts_series = SSTSSeries(self.path, require_ordering=require_ordering)

    def serial_ids(self):
        """
        Return serial ids from the model's SSTS series.

        Behavior:
        - Delegates to `self.ssts_series.serial_ids()`.
        """
        return self.ssts_series.serial_ids()

    def process(self, output_dir):
        """
        Build processed per-label NPY files for this model.

        Parameters:
        - `output_dir`: destination directory for processed `.npy` outputs.
        """
        return self.ssts_series.process(output_dir)
