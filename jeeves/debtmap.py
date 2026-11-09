"""Debtmap validation for `j lint`."""

import sh

from jeeves.repository import SH_CWD


def lint_debtmap() -> None:
    """Fail when project debt density exceeds the debtmap threshold."""
    sh.cargo(
        'test',
        '--locked',
        '--test',
        'debtmap',
        '--',
        '--nocapture',
        _cwd=SH_CWD,
        _fg=True,
    )
