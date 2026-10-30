"""Run documented scenarios through the public SPARQL endpoint."""

import shutil
from pathlib import Path

import pytest
from serving import (
    PROJECT_DIR,
    ROOT,
    SERVER_LOG_NAME,
    _serve,
    _wait_for_queries,
)

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


def test_recursive_context_example(sparqld_binary: Path, tmp_path: Path) -> None:
    """Query the unchanged documentation project directly."""
    with _serve(
        SCENARIO / PROJECT_DIR,
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as server:
        _wait_for_queries(server, QUERIES, SCENARIO / 'results/initial')


def test_declaration_removal_unregisters_context(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Delete the declaration in a temporary project and wait for the error state."""
    project = shutil.copytree(SCENARIO / PROJECT_DIR, tmp_path / PROJECT_DIR)
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(server, QUERIES, SCENARIO / 'results/initial')
        (project / 'remote.jsonld').unlink()
        _wait_for_queries(server, QUERIES, SCENARIO / 'results/unregistered')


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
    ) as server:
        _wait_for_queries(
            server,
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
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )
        shutil.copyfile(DECLARATIONS / f'{layout}.{extension}', declaration)
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
        shutil.copyfile(
            DECLARATIONS / f'{layout}-updated.{extension}',
            declaration,
        )
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/new',
        )
        shutil.copyfile(project / 'metadata.ttl', declaration)
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )


def test_context_registration_chain(sparqld_binary: Path, tmp_path: Path) -> None:
    """Resolve a declaration that itself needs another registered context."""
    with _serve(
        CONTEXT_RELOAD / 'chain',
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as server:
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)


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
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
        shutil.copyfile(
            DECLARATIONS / f'{layout}-updated.{extension}',
            project / f'conflict.{extension}',
        )
        _wait_for_queries(
            server,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results' / f'ambiguous-{layout}',
        )
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(
            server,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results' / f'ambiguous-{layout}',
        )
        (project / f'conflict.{extension}').unlink()
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
