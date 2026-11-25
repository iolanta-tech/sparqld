"""Start a local sparqld process for documentation captures."""

from __future__ import annotations

import socket
import time
from collections.abc import Generator
from contextlib import contextmanager, suppress
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen

import sh

WAIT_SECONDS = 10


class Server:
    """A running sparqld process and the endpoint that reaches it."""

    def __init__(self, endpoint: str, process: sh.RunningCommand) -> None:
        self.endpoint = endpoint
        self.process = process


def query(endpoint: str, query_path: Path) -> bytes:
    """POST a SPARQL query file and return the response body."""
    request = Request(
        endpoint,
        data=query_path.read_bytes(),
        headers={'Content-Type': 'application/sparql-query'},
        method='POST',
    )
    with urlopen(request, timeout=1) as response:
        return response.read()


@contextmanager
def serve(
    directory: Path,
    binary: Path,
    output_path: Path,
) -> Generator[Server]:
    """Run sparqld against `directory` until the caller exits the context."""
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    endpoint = f'http://127.0.0.1:{port}/'
    with output_path.open('w+') as output:
        try:
            process = sh.Command(str(binary))(  # noqa: TID251 - Use the caller's exact binary path.
                '--directory',
                str(directory),
                'http',
                '--port',
                str(port),
                _bg=True,
                _bg_exc=False,
                _out=output,
                _err_to_out=True,
            )
        except sh.CommandNotFound as error:
            raise OSError(f'Cannot execute sparqld at {binary}') from error
        try:
            yield Server(endpoint, process)
        finally:
            _stop(process)


def wait_until_ready(
    server: Server,
    queries: tuple[tuple[str, Path], ...],
    log: Path,
) -> None:
    """Block until every query returns HTTP success or the wait budget expires."""
    deadline = time.monotonic() + WAIT_SECONDS
    last_failure = ''
    while time.monotonic() < deadline:
        if not server.process.is_alive():
            detail = log.read_text()
            raise RuntimeError(f'sparqld exited before answering queries:\n{detail}')
        try:
            for _, query_path in queries:
                query(server.endpoint, query_path)
        except (OSError, URLError) as error:
            last_failure = str(error)
        else:
            return
        time.sleep(0.1)
    detail = log.read_text()
    raise RuntimeError(f'Timed out waiting for sparqld: {last_failure}\n{detail}')


def _stop(process: sh.RunningCommand) -> None:
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
            process.wait(timeout=5)
