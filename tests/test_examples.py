"""Run documented scenarios through the public SPARQL endpoint."""

import json
import shutil
from pathlib import Path

import pytest
from ruamel.yaml import YAML
from serving import (
    PROJECT_DIR,
    ROOT,
    SERVER_LOG_NAME,
    _serve,
    _wait_for_queries,
)

CONTEXT_RELOAD = ROOT / 'docs/examples/context-reload'
OLD_CONTEXT_RESULTS = CONTEXT_RELOAD / 'results/old'
RESULTS_DIR = 'results'
CONTEXT_RELOAD_QUERIES = tuple(
    (name, CONTEXT_RELOAD / 'queries' / f'{name}.rq')
    for name in ('alpha-centauri', 'failures')
)
CONFLICT_QUERIES = CONTEXT_RELOAD_QUERIES + (
    ('dependencies', ROOT / 'docs/queries/context-dependencies.rq'),
)
SCENARIO = ROOT / 'docs/examples/recursive-context'
QUERIES = (
    ('declaration', SCENARIO / 'queries/declaration.rq'),
    ('failures', SCENARIO / 'queries/failures.rq'),
    ('alpha-centauri', SCENARIO / 'queries/alpha-centauri.rq'),
    ('dependencies', ROOT / 'docs/queries/context-dependencies.rq'),
)
YAML_CODEC = YAML()
CONTEXT_URL = 'urn:context:second'


def _dump_yaml(path: Path, document: dict) -> None:
    with path.open('w') as stream:
        YAML_CODEC.dump(document, stream)


def _registered_body(name_term: str) -> dict:
    return {
        '@context': {'name': name_term},
        '@type': 'http://www.w3.org/ns/json-ld#Context',
        'http://schema.org/contentUrl': {'@id': CONTEXT_URL},
    }


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
    """Remove the registration in a temporary project and wait for the error state."""
    project = shutil.copytree(SCENARIO / PROJECT_DIR, tmp_path / PROJECT_DIR)
    body = project / 'example.jsonld'
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(server, QUERIES, SCENARIO / 'results/initial')
        document = json.loads(body.read_text())
        document.pop('http://schema.org/contentUrl')
        body.write_text(json.dumps(document))
        _wait_for_queries(server, QUERIES, SCENARIO / 'results/unregistered')


def test_default_context_needs_no_request(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Expand dollar aliases from the embedded default context with no request."""
    scenario = ROOT / 'docs/examples/default-context'
    log = tmp_path.joinpath(SERVER_LOG_NAME)
    with _serve(scenario / PROJECT_DIR, sparqld_binary, log) as server:
        _wait_for_queries(
            server,
            (('triples', scenario / 'queries/triples.rq'),),
            scenario / RESULTS_DIR,
        )


def test_context_registration_recovery_and_change(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Register, retarget, and unregister a context while generic metadata remains."""
    project = shutil.copytree(CONTEXT_RELOAD / PROJECT_DIR, tmp_path / PROJECT_DIR)
    body = project / 'body.yamlld'
    alternate = project / 'alternate.yamlld'
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )
        _dump_yaml(body, _registered_body('urn:old'))
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
        _dump_yaml(body, {'@context': {'name': 'urn:old'}})
        _dump_yaml(alternate, _registered_body('urn:new'))
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/new',
        )
        _dump_yaml(alternate, {'@context': {'name': 'urn:new'}})
        _wait_for_queries(
            server,
            CONTEXT_RELOAD_QUERIES,
            CONTEXT_RELOAD / 'results/unregistered',
        )


def test_conflicting_registration_matches_restart(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Invalidate both bodies and their consumer, then recover on deletion."""
    project = shutil.copytree(CONTEXT_RELOAD / PROJECT_DIR, tmp_path / PROJECT_DIR)
    body = project / 'body.yamlld'
    alternate = project / 'alternate.yamlld'
    _dump_yaml(body, _registered_body('urn:old'))
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
        _dump_yaml(alternate, _registered_body('urn:new'))
        _wait_for_queries(
            server,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results/ambiguous',
        )
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        _wait_for_queries(
            server,
            CONFLICT_QUERIES,
            CONTEXT_RELOAD / 'results/ambiguous',
        )
        alternate.unlink()
        _wait_for_queries(server, CONTEXT_RELOAD_QUERIES, OLD_CONTEXT_RESULTS)
