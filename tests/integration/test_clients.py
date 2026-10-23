"""SPARQL client checks against a live sparqld endpoint."""

# flake8: noqa: WPS210, WPS226

import shutil
import subprocess
import tempfile
from collections.abc import Iterator, Mapping, Sequence
from pathlib import Path

import pytest

from docs_macros import CLIENT_COMPATIBILITY_PATH
from mkdocs_macros_sparqld import ensure_endpoint, stop_server
from mkdocs_macros_sparqld.server import configure as configure_server

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / 'docs'
QUERIES = DOCS / 'queries'
DISPLAY_ENDPOINT = 'http://127.0.0.1:7737/'
ALPHA_CENTAURI = 'Alpha Centauri'
QUERY_FLAG = '-f'
GRAPH = 'sparqld:examples/alpha-centauri.yamlld'
SUMMARY = (
    'curl, sq, rsparql, Comunica, and sparqlquery passed live compatibility checks.'
)


def _executable(name: str, fallback: Path | None = None) -> str:
    found = shutil.which(name)
    if found:
        return found
    if fallback is not None and fallback.is_file():
        return str(fallback)
    raise RuntimeError(f'Required client `{name}` was not found.')


def _jena_rsparql():
    matches = sorted((ROOT / '.tools').glob('apache-jena-*/bin/rsparql'))
    fallback = matches[0] if matches else None
    return _executable('rsparql', fallback)


@pytest.fixture(scope='session')
def endpoint():
    """Start sparqld over the documentation directory."""
    configure_server(
        project_dir=ROOT,
        directory=DOCS.resolve(),
        binary='sparqld',
    )
    try:
        yield ensure_endpoint()
    finally:
        stop_server()


@pytest.fixture(scope='session')
def queries():
    """Paths of the shared example queries."""
    return {
        'select': str(QUERIES / 'names.rq'),
        'ask': str(QUERIES / 'ask-data.rq'),
        'construct': str(QUERIES / 'construct-name.rq'),
        'named': str(QUERIES / 'sq-named-graph.rq'),
    }


@pytest.fixture(scope='session')
def sq():
    """The sq executable."""
    return _executable('sq')


@pytest.fixture(scope='session')
def rsparql():
    """Jena's rsparql executable."""
    return _jena_rsparql()


@pytest.fixture(scope='session')
def comunica():
    """The Comunica command-line executable."""
    return _executable(
        'comunica-sparql',
        ROOT / 'node_modules' / '.bin' / 'comunica-sparql',
    )


@pytest.fixture(scope='session')
def sparqlquery():
    """RDFLib's sparqlquery executable."""
    return _executable('sparqlquery')


@pytest.fixture(scope='session')
def curl():
    """The curl executable."""
    return _executable('curl')


@pytest.fixture(scope='session')
def sq_directory(endpoint: str) -> Iterator[Path]:
    """Temporary sq configuration pointed at the live endpoint."""
    source = DOCS / 'reference/clients/sq/sq.toml'
    config = source.read_text().replace(DISPLAY_ENDPOINT, endpoint)
    with tempfile.TemporaryDirectory(prefix='sparqld-sq-') as directory:
        path = Path(directory)
        (path / '.sq.toml').write_text(config)
        yield path


@pytest.fixture(scope='session')
def compatibility(
    request: pytest.FixtureRequest,
) -> Iterator[list[tuple[str, str, str]]]:
    """Collect passed checks and write the YAML-LD record."""
    passed = []
    yield passed
    if request.session.testsfailed:
        return
    CLIENT_COMPATIBILITY_PATH.write_text(_document(passed))


def _arguments(
    template: Sequence[str],
    endpoint: str,
    queries: Mapping[str, str],
) -> list[str]:
    values = {'endpoint': endpoint, **queries}
    rendered = []
    for part in template:
        text = part
        for key, value in values.items():
            text = text.replace('{' + key + '}', value)
        rendered.append(text)
    return rendered


def _run(
    executable: str,
    arguments: Sequence[str],
    expected: str,
    cwd: Path | None,
) -> None:
    completed = subprocess.run(
        [executable, *arguments],
        capture_output=True,
        text=True,
        check=False,
        cwd=cwd,
    )
    assert completed.returncode == 0, completed.stderr
    assert expected in completed.stdout


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        ('graphs', ('-e', '{endpoint}', 'graphs'), GRAPH),
        ('ask', ('-e', '{endpoint}', QUERY_FLAG, '{ask}'), 'true'),
        (
            'construct',
            ('-e', '{endpoint}', QUERY_FLAG, '{construct}'),
            ALPHA_CENTAURI,
        ),
    ],
)
def test_sq(
    endpoint: str,
    queries: Mapping[str, str],
    sq: str,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """sq prints the expected response."""
    _run(sq, _arguments(arguments, endpoint, queries), expected, None)
    compatibility.append(('sq', check, expected))


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        (
            'select',
            ('--service', '{endpoint}', '--query', '{select}'),
            ALPHA_CENTAURI,
        ),
        ('ask', ('--service', '{endpoint}', '--query', '{ask}'), 'Yes'),
        (
            'construct',
            ('--service', '{endpoint}', '--query', '{construct}'),
            ALPHA_CENTAURI,
        ),
    ],
)
def test_rsparql(
    endpoint: str,
    queries: Mapping[str, str],
    rsparql: str,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """rsparql prints the expected response."""
    _run(rsparql, _arguments(arguments, endpoint, queries), expected, None)
    compatibility.append(('rsparql', check, expected))


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        ('select', ('sparql@{endpoint}', QUERY_FLAG, '{select}'), ALPHA_CENTAURI),
        ('ask', ('sparql@{endpoint}', QUERY_FLAG, '{ask}'), 'true'),
        (
            'construct',
            ('sparql@{endpoint}', QUERY_FLAG, '{construct}'),
            ALPHA_CENTAURI,
        ),
    ],
)
def test_comunica(
    endpoint: str,
    queries: Mapping[str, str],
    comunica: str,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """Comunica prints the expected response."""
    _run(comunica, _arguments(arguments, endpoint, queries), expected, None)
    compatibility.append(('comunica-sparql', check, expected))


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        ('select', ('{endpoint}', '--queryfile', '{select}'), ALPHA_CENTAURI),
        ('ask', ('{endpoint}', '--queryfile', '{ask}'), 'true'),
        (
            'construct',
            ('{endpoint}', '--queryfile', '{construct}'),
            ALPHA_CENTAURI,
        ),
    ],
)
def test_sparqlquery(
    endpoint: str,
    queries: Mapping[str, str],
    sparqlquery: str,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """sparqlquery prints the expected response."""
    _run(sparqlquery, _arguments(arguments, endpoint, queries), expected, None)
    compatibility.append(('sparqlquery', check, expected))


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        (
            'ask',
            (
                '--silent',
                '--show-error',
                '--fail-with-body',
                '--get',
                '{endpoint}',
                '--data-urlencode',
                'query=ASK { ?subject ?predicate ?object }',
            ),
            '"boolean":true',
        ),
    ],
)
def test_curl(
    endpoint: str,
    queries: Mapping[str, str],
    curl: str,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """curl prints the expected response."""
    _run(curl, _arguments(arguments, endpoint, queries), expected, None)
    compatibility.append(('curl', check, expected))


@pytest.mark.integration
@pytest.mark.parametrize(
    ('check', 'arguments', 'expected'),
    [
        ('graphs', ('graphs',), 'data:index.md'),
        ('named graph', (QUERY_FLAG, '{named}'), 'sparqld'),
    ],
)
def test_sq_configuration(
    queries: Mapping[str, str],
    sq: str,
    sq_directory: Path,
    compatibility: list[tuple[str, str, str]],
    check: str,
    arguments: Sequence[str],
    expected: str,
) -> None:
    """sq uses its configuration file for the endpoint."""
    _run(sq, _arguments(arguments, '', queries), expected, sq_directory)
    compatibility.append(('sq configuration', check, expected))


def _document(checks: list[tuple[str, str, str]]) -> str:
    grouped = {}
    order = []
    for client, check, expected in checks:
        if client not in grouped:
            order.append(client)
            grouped[client] = []
        grouped[client].append({'name': check, 'expected': expected})
    lines = [
        '"@context":',
        '  schema: https://schema.org/',
        '  name: schema:name',
        '  summary: schema:abstract',
        '  expected: schema:value',
        '"@id": results/client-compatibility.yamlld',
        '"@type": schema:ReviewAction',
        f'name: {_quoted("SPARQL client compatibility")}',
        f'summary: {_quoted(SUMMARY)}',
        'client:',
    ]
    for client in order:
        lines.append(f'  - name: {_quoted(client)}')
        lines.append('    check:')
        for check in grouped[client]:
            lines.append(f'      - name: {_quoted(check["name"])}')
            lines.append(f'        expected: {_quoted(check["expected"])}')
    lines.append('')
    return '\n'.join(lines)


def _quoted(literal: str) -> str:
    escaped = str(literal).replace('\\', r'\\').replace('"', r'\"')
    return f'"{escaped}"'
