"""Model cluster discovery and filtering."""

from pathlib import Path

from ..ssts import ValidationError, load_ssts_json
from .label_and_path_mapper import label_matches, path_to_label
from .model import Model


class ModelCluster:
    """
    A cluster of models discovered under one root directory.

    Fields:
    - `root_path`: discovery root.
    - `models`: discovered Model objects with mapped labels.
    - `require_ordering`: passed through when constructing Model instances.
    - `strict_label`: if True, duplicate-key conflicts in label parsing raise.
    """

    def __init__(self, root_path, require_ordering=False, strict_label=False):
        """
        Create and immediately scan a model cluster.

        Parameters:
        - `root_path`: root directory containing model directories.
        - `require_ordering`: ordering validation behavior for model series.
        - `strict_label`: strict mode for path-label parsing conflicts.
        """
        self.root_path = Path(root_path)
        self.require_ordering = require_ordering
        self.strict_label = strict_label
        self.models = []
        self.scan()

    def _iter_dirs(self):
        """
        Yield all candidate directories under root, including root itself.
        """
        if self.root_path.is_dir():
            yield self.root_path
        for path in self.root_path.rglob("*"):
            if path.is_dir():
                yield path

    def _iter_ssts_json_paths(self, directory):
        """
        Yield candidate SSTS JSON files in one directory.

        Behavior:
        - Non-recursive scan of `*.json`.
        - Excludes sidecars ending with `.scalars.json`.
        """
        for path in sorted(Path(directory).glob("*.json"), key=lambda p: p.name):
            if path.name.endswith(".scalars.json"):
                continue
            if path.is_file():
                yield path

    def _is_valid_model_dir(self, directory):
        """
        Check whether a directory qualifies as a model directory.

        Behavior:
        - Requires at least one candidate SSTS JSON file.
        - Every candidate file must be valid SSTS JSON.
        - Does not retain loaded payloads in memory.
        """
        json_paths = list(self._iter_ssts_json_paths(directory))
        if not json_paths:
            return False

        for path in json_paths:
            try:
                load_ssts_json(path, require_ordering=self.require_ordering)
            except (ValidationError, OSError, ValueError):
                return False
        return True

    def scan(self):
        """
        Discover all valid models under root and rebuild model index.

        Returns:
        - `self` with refreshed `models` list.
        """
        models = []
        for directory in self._iter_dirs():
            if not self._is_valid_model_dir(directory):
                continue

            label = path_to_label(self.root_path, directory, strict=self.strict_label)
            model = Model(
                path=directory,
                label=label,
                require_ordering=self.require_ordering,
            )
            models.append(model)

        models.sort(key=lambda m: (str(m.path), sorted(m.label.items())))
        self.models = models
        return self

    def labels(self):
        """
        Return all discovered model labels.
        """
        return [dict(model.label) for model in self.models]

    def paths(self):
        """
        Return all discovered model directory paths.
        """
        return [model.path for model in self.models]

    def filter(self, **filters):
        """
        Return all models matching key-value filters.

        Parameters:
        - `filters`: key-value pairs that must match model label entries.
        """
        return [model for model in self.models if label_matches(model.label, filters)]

    def get(self, **filters):
        """
        Return one model matching key-value filters.

        Parameters:
        - `filters`: key-value pairs that must match model label entries.

        Behavior:
        - Raises KeyError when no model matches.
        - Raises ValueError when more than one model matches.
        """
        matches = self.filter(**filters)
        if not matches:
            raise KeyError(f"No model matched filters: {filters}")
        if len(matches) > 1:
            raise ValueError(f"Multiple models matched filters: {filters}")
        return matches[0]
