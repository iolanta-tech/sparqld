"""Compare context bodies that carry their registration with separate declarations."""

import select
import shutil
import socket
import subprocess
import time
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

import pytest
from ruamel.yaml import YAML
from test_examples import ROOT, WAIT_SECONDS, _query, _serve, _wait_for_queries

from mkdocs_macros_sparqld.render import SelectResult

SCENARIO = ROOT / 'docs/examples/self-registering-context'
CANONICAL = 'https://example.org/contexts/combined/person'
BODY = 'contexts/person.yamlld'
DOLLAR_BODY = 'contexts/z-dollar.yamlld'
REGISTRATION = 'contexts/z-registration.yamlld'
LAYOUTS = ('combined', 'separate')
YAML_CODEC = YAML()
PANIC = 'The remote context stack is invalid'


class ContextStackPanic(RuntimeError):
    """A verified oxjsonld failure rather than a generic endpoint timeout."""


KNOWN_PANIC = pytest.mark.xfail(
    strict=True,
    raises=ContextStackPanic,
    reason='oxjsonld 0.2.5 context stack assertion',
)
QUERIES = tuple(
    (name, SCENARIO / 'queries' / f'{name}.rq')
    for name in ('alice', 'failures', 'registration')
)


def _write(path: Path, document: dict) -> None:
    with path.open('w') as stream:
        YAML_CODEC.dump(document, stream)


def _read(path: Path) -> dict:
    return YAML_CODEC.load(path.read_text())


def _wait_state(
    endpoint: str,
    process: subprocess.Popen,
    names: list[str],
    failures: list[str],
    registered: bool = True,
) -> None:
    deadline = time.monotonic() + WAIT_SECONDS
    actual = []
    expected = [
        [{'name': name} for name in names],
        [{'file': f'sparqld:{path}'} for path in sorted(failures)],
        registered,
    ]
    while time.monotonic() < deadline:
        assert process.poll() is None
        try:
            responses = [_query(endpoint, query) for _, query in QUERIES]
            actual = [
                response.rows()
                if isinstance(response, SelectResult)
                else response.boolean
                for response in responses
            ]
            if actual == expected:
                return
        except OSError:
            pass
        time.sleep(0.1)
    pytest.fail(f'Expected {expected}; got {actual}')


@contextmanager
def _recognize_panic(log: Path) -> Generator[None]:
    try:
        yield
    except pytest.fail.Exception:
        if PANIC in log.read_text():
            raise ContextStackPanic(log.read_text()) from None
        raise


@pytest.mark.parametrize('layout', LAYOUTS)
def test_bootstrap(sparqld_binary: Path, tmp_path: Path, layout: str) -> None:
    """Late dollar registration and late person registration resolve on startup."""
    with _serve(SCENARIO / layout, sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_for_queries(endpoint, process, QUERIES, SCENARIO / 'results')


@pytest.mark.parametrize(
    ('layout', 'change'),
    [
        pytest.param(layout, change, marks=KNOWN_PANIC)
        if (layout, change) == ('separate', 'dollar')
        else (layout, change)
        for layout in LAYOUTS
        for change in ('edit', 'delete', 'invalid', 'unregister', 'dollar')
    ],
)
def test_reload_and_restart(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
    change: str,
) -> None:
    """Live changes and cold starts reach the same state; restoring files recovers."""
    project = shutil.copytree(SCENARIO / layout, tmp_path / 'project')
    declaration = BODY if layout == 'combined' else REGISTRATION
    target = project / (DOLLAR_BODY if change == 'dollar' else BODY)
    if change == 'unregister':
        target = project / declaration
    original = target.read_text()
    failures = []
    names = []
    registered = layout == 'separate'
    with _serve(project, sparqld_binary, tmp_path / 'live.log') as (endpoint, process):
        _wait_state(endpoint, process, ['Alice'], [])
        if change == 'edit':
            document = _read(target)
            document['@context'][1]['name'] = 'https://schema.org/alternateName'
            _write(target, document)
            registered = True
        elif change == 'unregister':
            document = _read(target)
            del document['context-file']
            _write(target, document)
            failures = ['alice.yamlld']
            registered = False
        elif change == 'delete':
            target.unlink()
            failures = ['alice.yamlld']
        elif change == 'invalid':
            target.write_text('"@context": [')
            failures = ['alice.yamlld', BODY]
        else:
            target.unlink()
            failures = ['alice.yamlld', BODY]
        with _recognize_panic(tmp_path / 'live.log'):
            _wait_state(endpoint, process, names, failures, registered)
    with _serve(project, sparqld_binary, tmp_path / 'restart.log') as (
        endpoint,
        process,
    ):
        _wait_state(endpoint, process, names, failures, registered)
        target.write_text(original)
        _wait_state(endpoint, process, ['Alice'], [])


@pytest.mark.parametrize(
    'cycle', ('self', 'mutual', pytest.param('recursive', marks=KNOWN_PANIC))
)
def test_cycles_fail_without_hanging(
    sparqld_binary: Path,
    tmp_path: Path,
    cycle: str,
) -> None:
    """Registration bootstrap cycles and context-reference recursion fail finitely."""
    project = shutil.copytree(SCENARIO / 'combined', tmp_path / 'project')
    document = _read(project / BODY)
    if cycle == 'self':
        document['@context'][0] = CANONICAL
    elif cycle == 'mutual':
        dollar = _read(project / DOLLAR_BODY)
        dollar['@context'] = [CANONICAL, dollar['@context']]
        _write(project / DOLLAR_BODY, dollar)
    else:
        document['@context'].insert(0, 'person.yamlld')
    _write(project / BODY, document)
    failures = ['alice.yamlld', BODY]
    if cycle == 'mutual':
        failures.append(DOLLAR_BODY)
    with (
        _serve(project, sparqld_binary, tmp_path / 'server.log') as (endpoint, process),
        _recognize_panic(tmp_path / 'server.log'),
    ):
        _wait_state(endpoint, process, [], failures, False)


@pytest.mark.parametrize('layout', LAYOUTS)
def test_relative_context_uses_requested_uri(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """A source-relative import needs a canonical registration when reused remotely."""
    project = shutil.copytree(SCENARIO / layout, tmp_path / 'project')
    document = _read(project / BODY)
    terms = document['@context'][1]
    document['@context'][1] = {'@import': 'terms.yamlld'}
    _write(project / BODY, document)
    _write(project / 'contexts/terms.yamlld', {'@context': terms})
    with _serve(project, sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_state(endpoint, process, [], ['alice.yamlld'])
        _write(
            project / 'terms-registration.yamlld',
            {
                '@id': f'https://example.org/contexts/{layout}/terms.yamlld',
                'https://sparqld.iolanta.tech/context-file': {
                    '@id': 'sparqld:contexts/terms.yamlld',
                },
            },
        )
        _wait_state(endpoint, process, ['Alice'], [])


def test_aliases_inside_term_definitions_fail(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """The imported dollar aliases do not replace term-definition keywords."""
    project = shutil.copytree(SCENARIO / 'combined', tmp_path / 'project')
    document = _read(project / BODY)
    definition = document['@context'][1]['context-file']
    definition['$id'] = definition.pop('@id')
    _write(project / BODY, document)
    with _serve(project, sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_state(endpoint, process, [], ['alice.yamlld', BODY], False)


@pytest.mark.parametrize('registered', (False, True))
def test_http_context_stays_offline(
    sparqld_binary: Path,
    tmp_path: Path,
    registered: bool,
) -> None:
    """HTTP context URIs use project mappings or fail without contacting a listener."""
    project = tmp_path / 'project'
    project.mkdir()
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        listener.listen()
        port = listener.getsockname()[1]
        _write(
            project / 'alice.yamlld',
            {
                '@context': f'http://127.0.0.1:{port}/context',
                '@id': 'urn:alice',
                'name': 'Alice',
            },
        )
        if registered:
            _write(
                project / 'context.yamlld',
                {
                    '@context': {'name': 'https://schema.org/name'},
                    '@id': f'http://127.0.0.1:{port}/context',
                    'https://sparqld.iolanta.tech/context-file': {
                        '@id': 'context.yamlld'
                    },
                },
            )
        with _serve(project, sparqld_binary, tmp_path / 'server.log') as (
            endpoint,
            process,
        ):
            _wait_state(
                endpoint,
                process,
                ['Alice'] if registered else [],
                [] if registered else ['alice.yamlld'],
                False,
            )
            assert not select.select([listener], [], [], 0)[0]
    if not registered:
        assert 'context is not registered' in (tmp_path / 'server.log').read_text()


@KNOWN_PANIC
def test_missing_nested_dependency_on_startup(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """A missing nested context should produce catalog errors during a cold start."""
    project = shutil.copytree(SCENARIO / 'separate', tmp_path / 'project')
    (project / DOLLAR_BODY).unlink()
    with (
        _serve(project, sparqld_binary, tmp_path / 'server.log') as (endpoint, process),
        _recognize_panic(tmp_path / 'server.log'),
    ):
        _wait_state(endpoint, process, [], ['alice.yamlld', BODY])


@pytest.mark.parametrize('layout', LAYOUTS)
def test_explicit_base_changes_registration_target(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """An explicit document base overrides the source-directory registration base."""
    project = shutil.copytree(SCENARIO / layout, tmp_path / 'project')
    target = project / (BODY if layout == 'combined' else REGISTRATION)
    document = _read(target)
    context = document['@context']
    (context[1] if isinstance(context, list) else context)['@base'] = (
        'https://example.org/'
    )
    _write(target, document)
    with _serve(project, sparqld_binary, tmp_path / 'server.log') as (
        endpoint,
        process,
    ):
        _wait_state(endpoint, process, [], ['alice.yamlld'], False)
    assert 'context-file is not a local file' in (tmp_path / 'server.log').read_text()


@pytest.mark.parametrize('layout', LAYOUTS)
def test_registration_must_be_selected(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """Excluded bodies remain readable as contexts, but contribute no registration."""
    project = shutil.copytree(SCENARIO / layout, tmp_path / 'project')
    declaration = BODY if layout == 'combined' else REGISTRATION
    with _serve(
        project,
        sparqld_binary,
        tmp_path / 'server.log',
        arguments=('--pattern', f'!{declaration}'),
    ) as (endpoint, process):
        _wait_state(endpoint, process, [], ['alice.yamlld'], False)
        document = _read(project / 'alice.yamlld')
        document['@context'] = BODY
        _write(project / 'alice.yamlld', document)
        _wait_state(endpoint, process, ['Alice'], [], False)
