"""Process helpers for build-time sparqld servers."""

from __future__ import annotations

import shutil
import socket
import time
from collections.abc import Callable
from contextlib import suppress
from pathlib import Path
from urllib.request import urlopen

import sh

_STARTUP_TIMEOUT_SECONDS = 30
_PROBE_TIMEOUT_SECONDS = 0.25
_HTTP_OK = 200


def resolve_binary(project_directory: Path, binary: str | Path) -> str:
    candidate = Path(binary)
    if candidate.is_absolute() and candidate.is_file():
        return str(candidate)
    relative = (project_directory / candidate).resolve()
    if relative.is_file():
        return str(relative)
    found = shutil.which(str(binary))
    if found:
        return found
    raise RuntimeError(
        f'sparqld binary `{binary}` was not found. Install sparqld on PATH or set extra.sparqld.binary to an executable path.',
    )


def unused_port() -> int:
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def endpoint_ready(endpoint: str) -> bool:
    with urlopen(endpoint, timeout=_PROBE_TIMEOUT_SECONDS) as response:
        return response.status == _HTTP_OK


def stop_process(process: sh.RunningCommand) -> None:
    if not process.is_alive():
        return
    with suppress(ProcessLookupError):
        process.terminate()
    with suppress(sh.ErrorReturnCode):
        try:
            process.wait(timeout=5)
        except sh.TimeoutException:
            with suppress(ProcessLookupError):
                process.kill()
            process.wait()


def _startup_failure(process: sh.RunningCommand) -> RuntimeError:
    with suppress(sh.ErrorReturnCode):
        process.wait()
    return RuntimeError(
        'Could not start sparqld for documentation macros.\n'
        + (process.stderr.decode().strip() or process.stdout.decode().strip())
    )


def wait_until_ready(
    process: sh.RunningCommand,
    endpoint: str,
    *,
    on_failure: Callable[[], None],
) -> str:
    deadline = time.monotonic() + _STARTUP_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        if not process.is_alive():
            on_failure()
            raise _startup_failure(process)
        try:
            if endpoint_ready(endpoint):
                return endpoint
        except OSError:
            time.sleep(0.1)
            continue
        time.sleep(0.1)
    on_failure()
    raise RuntimeError('Timed out starting sparqld for documentation macros.')


def start_sparqld(
    *,
    binary: str,
    directory: Path | None,
    project_directory: Path,
    port: int,
) -> tuple[sh.RunningCommand, str]:
    endpoint = f'http://127.0.0.1:{port}/'
    source = () if directory is None else ('--directory', str(directory))
    try:
        process = sh.Command(binary)(  # noqa: TID251 - Use the resolved executable path.
            *source,
            'http',
            '--host',
            '127.0.0.1',
            '--port',
            str(port),
            '--no-watch',
            _cwd=str(project_directory),
            _bg=True,
            _bg_exc=False,
            _tty_out=False,
        )
    except sh.CommandNotFound as error:
        raise OSError(f'Cannot execute sparqld at {binary}') from error
    return process, endpoint
