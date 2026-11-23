"""Start sparqld and query its SPARQL endpoint."""

import socket
import subprocess
import time
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path
from urllib.request import Request, urlopen

import pytest
from pydantic import TypeAdapter

from mkdocs_macros_sparqld.render import AskResult, SelectResult

ROOT = Path(__file__).resolve().parents[1]
PROJECT_DIR = 'project'
SERVER_LOG_NAME = 'server.log'
WAIT_SECONDS = 10
QUERY_RESULT = TypeAdapter(AskResult | SelectResult)


class Server:
    """A running sparqld process and the endpoint that reaches it."""

    def __init__(self, endpoint: str, process: subprocess.Popen) -> None:
        """Store the endpoint URL and the process that serves it."""
        self.endpoint = endpoint
        self.process = process


def _query(endpoint: str, query: Path) -> AskResult | SelectResult:
    request = Request(
        endpoint,
        data=query.read_bytes(),
        headers={'Content-Type': 'application/sparql-query'},
        method='POST',
    )
    with urlopen(request, timeout=1) as response:
        return QUERY_RESULT.validate_json(response.read())


def _wait_for_queries(
    server: Server,
    queries: tuple[tuple[str, Path], ...],
    result_directory: Path,
) -> None:
    deadline = time.monotonic() + WAIT_SECONDS
    last_failure = ''
    while time.monotonic() < deadline:
        assert server.process.poll() is None, (
            'sparqld exited before reaching the expected state'
        )
        try:
            for name, query in queries:
                expected = QUERY_RESULT.validate_json(
                    (result_directory / name / 'results.json').read_bytes(),
                )
                assert _query(server.endpoint, query) == expected, query.name
        except (OSError, AssertionError) as error:
            last_failure = str(error)
        else:
            return
        time.sleep(0.1)
    pytest.fail(f'Timed out waiting for the example results: {last_failure}')


@contextmanager
def _serve(
    directory: Path,
    binary: Path,
    output_path: Path,
    arguments: tuple[str, ...] = (),
) -> Generator[Server]:
    port = _available_port()
    with _launch(
        binary,
        output_path,
        port,
        (str(directory), '--port', str(port), *arguments),
    ) as server:
        yield server


def _available_port() -> int:
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


@contextmanager
def _launch(
    binary: Path,
    output_path: Path,
    port: int,
    arguments: tuple[str, ...],
    working_directory: Path | None = None,
) -> Generator[Server]:
    endpoint = f'http://127.0.0.1:{port}/'
    with output_path.open('w+') as output:
        process = subprocess.Popen(
            [str(binary), *arguments],
            cwd=working_directory,
            stdout=output,
            stderr=output,
            text=True,
        )

        try:
            yield Server(endpoint, process)
        except (AssertionError, pytest.fail.Exception) as error:
            output.seek(0)
            pytest.fail(f'{error}\n{output.read()}')
        finally:
            _stop(process)


def _stop(process: subprocess.Popen) -> None:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
