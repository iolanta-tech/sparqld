"""Shared process helpers for project commands."""

from concurrent.futures import ThreadPoolExecutor

import sh

from jeeves.repository import SH_CWD


def in_parallel(rust_task, python_task):
    """Run the Rust and Python tasks. Return whether formatting changed files."""
    with ThreadPoolExecutor(max_workers=2) as executor:
        tasks = (
            executor.submit(rust_task),
            executor.submit(python_task),
        )
        changed = False
        for task in tasks:
            changed = task.result() or changed
        return changed


def output_text(stream):
    """Return a command stream as text."""
    if not stream:
        return ''
    if isinstance(stream, bytes):
        return stream.decode()
    return stream


def command_output(command, *arguments):
    """Run a command and return its combined output, including failures."""
    try:
        completed = command(*arguments, _cwd=SH_CWD, _tty_out=False)
    except sh.ErrorReturnCode as error:
        return output_text(error.stdout) + output_text(error.stderr)
    return str(completed)
