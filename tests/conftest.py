"""Shared fixtures for executable documentation examples."""

from pathlib import Path

import pytest
import sh


@pytest.fixture(scope='session')
def sparqld_binary() -> Path:
    """Build the current checkout before testing its public interface."""
    root = Path(__file__).resolve().parents[1]
    target_directory = root / 'target'
    sh.cargo.build(
        '--locked',
        '--target-dir',
        str(target_directory),
        _cwd=str(root),
    )
    return target_directory / 'debug/sparqld'
