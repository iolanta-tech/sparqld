"""Pluglet release commands."""

import sh

from jeeves.repository import SH_CWD


def publish_pluglet():
    """Build a clean pluglet distribution and publish it."""
    sh.poetry('build', '--clean', _cwd=SH_CWD, _fg=True)
    sh.poetry('publish', _cwd=SH_CWD, _fg=True)
