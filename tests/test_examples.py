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
PROJECT_DIR = 'project'
SERVER_LOG_NAME = 'server.log'
CONTEXT_RELOAD = ROOT / 'docs/examples/context-reload'
DECLARATIONS = CONTEXT_RELOAD / 'declarations'
OLD_CONTEXT_RESULTS = CONTEXT_RELOAD / 'results/old'
STANDALONE_LAYOUT = 'standalone'
CONTEXT_RELOAD_QUERIES = tuple(
    (name, CONTEXT_RELOAD / 'queries' / f'{name}.rq') for name in ('alice', 'failures')
)
CONFLICT_QUERIES = CONTEXT_RELOAD_QUERIES + (
    ('dependencies', ROOT / 'docs/queries/context-dependencies.rq'),
)
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


def _wait_for_queries(
    endpoint: str,
    process: subprocess.Popen,
    queries: tuple[tuple[str, Path], ...],
    result_directory: Path,
) -> None:
    deadline = time.monotonic() + WAIT_SECONDS
    last_failure = ''
    while time.monotonic() < deadline:
        assert process.poll() is None, (
            'sparqld exited before reaching the expected state'
        )
        try:
            for name, query in queries:
                expected = QUERY_RESULT.validate_json(
                    (result_directory / f'{name}.json').read_bytes(),
                )
                assert _query(endpoint, query) == expected, query.name
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
                *arguments,
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
    with _serve(
        SCENARIO / PROJECT_DIR,
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as (endpoint, process):
        _wait_for_queries(
            endpoint,
            process,
            QUERIES,
            SCENARIO / 'results/initial',
        )


def test_declaration_removal_unregisters_context(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Delete the declaration in a temporary project and wait for the error state."""
    project = shutil.copytree(SCENARIO / PROJECT_DIR, tmp_path / PROJECT_DIR)
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as (
        endpoint,
        process,
    ):
        _wait_for_queries(
            endpoint,
            process,
            QUERIES,
            SCENARIO / 'results/initial',
        )
        (project / 'remote.jsonld').unlink()
        _wait_for_queries(
            endpoint,
            process,
            QUERIES,
            SCENARIO / 'results/unregistered',
        )


def test_registered_yaml_context_example(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Query the YAML-LD registration example through the public endpoint."""
    scenario = ROOT / 'docs/examples/registered-context'
    with _serve(
        scenario / PROJECT_DIR,
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as (endpoint, process):
        _wait_for_queries(
            endpoint,
            process,
            (('alice', scenario / 'queries/alice.rq'),),
            scenario / 'results',
        )


@pytest.mark.parametrize('layout', [STANDALONE_LAYOUT, 'embedded'])
def test_context_registration_recovery_and_change(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """Register, retarget, and unregister a context while generic metadata remains."""
    project = shutil.copytree(CONTEXT_RELOAD / PROJECT_DIR, tmp_path / PROJECT_DIR)
    extension = 'ttl' if layout == STANDALONE_LAYOUT else 'trig'
    declaration = project / f'declaration.{extension}'
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as (
        endpoint,
        process,
    ):
        _wait_for_queries(
            endpoint,
            process,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )
        shutil.copyfile(
            DECLARATIONS / f'{layout}.{extension}',
            declaration,
        )
        _wait_for_queries(
            endpoint, process, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS
        )
        shutil.copyfile(
            DECLARATIONS / f'{layout}-updated.{extension}',
            declaration,
        )
        _wait_for_queries(
            endpoint, process, CONTEXT_RELOAD_QUERIES, CONTEXT_RELOAD / 'results/new'
        )
        shutil.copyfile(project / 'metadata.ttl', declaration)
        _wait_for_queries(
            endpoint,
            process,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )


def test_context_registration_chain(sparqld_binary: Path, tmp_path: Path) -> None:
    """Resolve a declaration that itself needs another registered context."""
    with _serve(
        CONTEXT_RELOAD / 'chain', sparqld_binary, tmp_path / SERVER_LOG_NAME
    ) as (
        endpoint,
        process,
    ):
        _wait_for_queries(
            endpoint, process, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS
        )


@pytest.mark.parametrize('layout', [STANDALONE_LAYOUT, 'embedded'])
def test_conflicting_registration_matches_restart(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """Invalidate both declarations and their consumer, then recover on deletion."""
    project = shutil.copytree(CONTEXT_RELOAD / PROJECT_DIR, tmp_path / PROJECT_DIR)
    extension = 'ttl' if layout == STANDALONE_LAYOUT else 'trig'
    shutil.copyfile(
        DECLARATIONS / f'{layout}.{extension}',
        project / f'declaration.{extension}',
    )
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as (
        endpoint,
        process,
    ):
        _wait_for_queries(
            endpoint, process, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS
        )
        shutil.copyfile(
            DECLARATIONS / f'{layout}-updated.{extension}',
            project / f'conflict.{extension}',
        )
        _wait_for_queries(
            endpoint,
            process,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results' / f'ambiguous-{layout}',
        )
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as (
        endpoint,
        process,
    ):
        _wait_for_queries(
            endpoint,
            process,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results' / f'ambiguous-{layout}',
        )
        (project / f'conflict.{extension}').unlink()
        _wait_for_queries(
            endpoint, process, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS
        )
