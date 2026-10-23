"""Pluglet release commands."""

import sh

from jeeves.repository import PROJECT_ROOT, SH_CWD


def publish_pluglet():
    """Build a clean pluglet distribution and publish it."""
    poetry = sh.Command(str(PROJECT_ROOT / '.venv/bin/poetry'))
    poetry('build', '--clean', _cwd=SH_CWD, _fg=True)
    poetry('publish', _cwd=SH_CWD, _fg=True)
