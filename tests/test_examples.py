"""Run documented scenarios through the public SPARQL endpoint."""

import shutil
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
SCENARIO = ROOT / 'docs/examples/recursive-context'
QUERIES = (
    ('declaration', SCENARIO / 'queries/declaration.rq'),
    ('failures', SCENARIO / 'queries/failures.rq'),
    ('alice', SCENARIO / 'queries/alice.rq'),
    ('dependencies', ROOT / 'docs/queries/context-dependencies.rq'),
)
QUERY_RESULT = TypeAdapter(AskResult | SelectResult)
WAIT_SECONDS = 10


def _query(endpoint: str, query: Path) -> AskResult | SelectResult:
    request = Request(
        endpoint,
        data=query.read_bytes(),
        headers={'Content-Type': 'application/sparql-query'},
        method='POST',
    )
    with urlopen(request, timeout=1) as response:
        return QUERY_RESULT.validate_json(response.read())


def _assert_state(endpoint: str, state: str) -> None:
    for name, query in QUERIES:
        expected = QUERY_RESULT.validate_json(
            (SCENARIO / 'results' / state / f'{name}.json').read_bytes(),
        )
        assert _query(endpoint, query) == expected, f'{state}: {query.name}'


def _wait_for_state(
    endpoint: str,
    state: str,
    process: subprocess.Popen,
) -> None:
    deadline = time.monotonic() + WAIT_SECONDS
    last_failure = ''
    while time.monotonic() < deadline:
        assert process.poll() is None, (
            'sparqld exited before reaching the expected state'
        )
        try:
            _assert_state(endpoint, state)
        except (OSError, AssertionError) as error:
            last_failure = str(error)
        else:
            return
        time.sleep(0.1)
    pytest.fail(f'Timed out waiting for the {state} example results: {last_failure}')


@contextmanager
def _serve(
    directory: Path,
    binary: Path,
    output_path: Path,
) -> Generator[tuple[str, subprocess.Popen]]:
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    endpoint = f'http://127.0.0.1:{port}/'
    with output_path.open('w+') as output:
        process = subprocess.Popen(
            [
                str(binary),
                str(directory),
                '--port',
                str(port),
            ],
            stdout=output,
            stderr=output,
            text=True,
        )
        try:
            yield endpoint, process
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


def test_recursive_context_example(sparqld_binary: Path, tmp_path: Path) -> None:
    """Query the unchanged documentation project directly."""
    with _serve(SCENARIO / 'project', sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_for_state(endpoint, 'initial', process)


def test_recursive_context_recovery(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Modify a temporary project and verify its documented recovered state."""
    project = shutil.copytree(SCENARIO / 'project', tmp_path / 'project')
    with _serve(project, sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_for_state(endpoint, 'initial', process)
        shutil.copyfile(
            SCENARIO / 'recovered-context.jsonld', project / 'context.jsonld'
        )
        _wait_for_state(endpoint, 'recovered', process)
