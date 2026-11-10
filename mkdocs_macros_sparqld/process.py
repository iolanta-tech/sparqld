"""Subprocess helpers for build-time sparqld servers."""

from __future__ import annotations

import shutil
import socket
import subprocess
import time
from collections.abc import Callable
from pathlib import Path
from urllib.request import urlopen

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


def stop_process(process: subprocess.Popen) -> None:
    if process.poll() is not None:
        return
    process.terminate()
    try:
        process.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.communicate()


def _startup_failure(process: subprocess.Popen) -> RuntimeError:
    stdout, stderr = process.communicate()
    return RuntimeError(
        'Could not start sparqld for documentation macros.\n'
        + (stderr.strip() or stdout.strip())
    )


def wait_until_ready(
    process: subprocess.Popen,
    endpoint: str,
    *,
    on_failure: Callable[[], None],
) -> str:
    deadline = time.monotonic() + _STARTUP_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        if process.poll() is not None:
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
    directory: Path,
    project_directory: Path,
    port: int,
) -> tuple[subprocess.Popen, str]:
    endpoint = f'http://127.0.0.1:{port}/'
    process = subprocess.Popen(
        [
            binary,
            str(directory),
            '--host',
            '127.0.0.1',
            '--port',
            str(port),
            '--no-watch',
        ],
        cwd=project_directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return process, endpoint
