"""Project directory preparation for example result captures."""

from __future__ import annotations

import shutil
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class UnchangedProject:
    """Serve an existing project directory without copying it."""

    source: Path

    def __call__(self, _temporary: Path) -> Path:
        return self.source


@dataclass(frozen=True)
class CopiedProject:
    """Copy a project, optionally mutate it, then serve the copy."""

    source: Path
    mutate: Callable[[Path], None] | None = None

    def __call__(self, temporary: Path) -> Path:
        project = shutil.copytree(self.source, temporary / self.source.name)
        if self.mutate is not None:
            self.mutate(project)
        return project
