"""MkDocs macros for project documentation."""

import atexit
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from datetime import date
from pathlib import Path
from urllib.request import Request, urlopen


ROOT_DIR = Path(__file__).resolve().parent
EXAMPLES_DIR = ROOT_DIR / 'docs' / 'examples'
CLIENTS_DIR = ROOT_DIR / 'docs' / 'reference' / 'clients'
LIBRARY_EXAMPLES_DIR = ROOT_DIR / 'docs' / 'reference' / 'libraries'
QUERIES_DIR = ROOT_DIR / 'docs' / 'queries'
RESULTS_DIR = ROOT_DIR / 'docs' / 'results'
REPO_URL = 'https://github.com/iolanta-tech/sparqld'
DISPLAY_ENDPOINT = 'http://127.0.0.1:7737/'


_docs_server = None
_docs_endpoint = None


_ADR_STATUS = {
    'draft': 'Draft',
    'undecided': 'Undecided',
    'decided': 'Decided',
}


_ADR_STATUS_ADMONITION = {
    'draft': 'note',
    'undecided': 'warning',
    'decided': 'success',
}


_EXAMPLE_ICONS = {
    '.json': ':material-code-json:',
    '.jsonld': ':material-code-json:',
    '.md': ':material-language-markdown:',
    '.rq': ':material-database-search-outline:',
    '.sh': ':material-console:',
    '.yaml': ':simple-yaml:',
    '.yamlld': ':simple-yaml:',
    '.yml': ':simple-yaml:',
}


_EXAMPLE_SYNTAXES = {
    '.json': 'json',
    '.jsonld': 'json',
    '.md': 'markdown',
    '.rq': 'sparql',
    '.sh': 'console',
    '.toml': 'toml',
    '.tsv': 'text',
    '.txt': 'text',
    '.yaml': 'yaml',
    '.yamlld': 'yaml',
    '.yml': 'yaml',
}


def _example_path(name):
    path = (EXAMPLES_DIR / name).resolve()
    if not path.is_relative_to(EXAMPLES_DIR.resolve()):
        raise ValueError(f'Invalid example path: {name}')
    return path


def _query_path(name):
    path = (QUERIES_DIR / name).resolve()
    if not path.is_relative_to(QUERIES_DIR.resolve()):
        raise ValueError(f'Invalid query path: {name}')
    return path


def _github_url(relative, repo_url, directory=False):
    relative = Path(relative).as_posix()
    operation = 'tree' if directory else 'blob'
    return f'{repo_url.rstrip("/")}/{operation}/main/{relative}'


def _indent_block(body, indent):
    pad = ' ' * indent
    return '\n'.join(f'{pad}{line}' for line in body.splitlines())


def _command(name, fallback=None):
    executable = shutil.which(name)
    if executable:
        return executable
    if fallback and fallback.is_file():
        return str(fallback)
    raise RuntimeError(
        f'Required documentation client `{name}` was not found. '
        'Install it and make it available on PATH before building the documentation.'
    )


def _run(command, expected=None, environment=None, cwd=ROOT_DIR):
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd,
            env=environment,
        )
    except FileNotFoundError as error:
        _stop_docs_server()
        raise RuntimeError(
            f'Required documentation command `{command[0]}` was not found.'
        ) from error
    except subprocess.CalledProcessError as error:
        _stop_docs_server()
        detail = (error.stderr or error.stdout).strip()
        raise RuntimeError(
            f'Documentation command failed: {" ".join(map(str, command))}\n{detail}'
        ) from error
    output = result.stdout.strip()
    if expected and expected not in output:
        _stop_docs_server()
        raise RuntimeError(
            f'Documentation command did not return `{expected}`: '
            f'{" ".join(map(str, command))}\n{output}'
        )
    return output


def _unused_port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def _ensure_docs_server():
    global _docs_endpoint, _docs_server
    if _docs_server is not None and _docs_server.poll() is None:
        return _docs_endpoint

    port = _unused_port()
    endpoint = f'http://127.0.0.1:{port}/'
    _docs_server = subprocess.Popen(
        [
            _command('cargo'),
            'run',
            '--quiet',
            '--',
            str(EXAMPLES_DIR),
            '--host',
            '127.0.0.1',
            '--port',
            str(port),
            '--no-watch',
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    _docs_endpoint = endpoint

    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if _docs_server.poll() is not None:
            stdout, stderr = _docs_server.communicate()
            _docs_server = None
            _docs_endpoint = None
            raise RuntimeError(
                'Could not start sparqld for documentation examples.\n'
                f'{stderr.strip() or stdout.strip()}'
            )
        try:
            with urlopen(endpoint, timeout=0.25) as response:
                if response.status == 200:
                    return endpoint
        except OSError:
            time.sleep(0.1)

    _stop_docs_server()
    raise RuntimeError('Timed out starting sparqld for documentation examples.')


def _stop_docs_server():
    global _docs_endpoint, _docs_server
    if _docs_server is None:
        return
    if _docs_server.poll() is None:
        _docs_server.terminate()
        try:
            _docs_server.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            _docs_server.kill()
            _docs_server.communicate()
    _docs_server = None
    _docs_endpoint = None


atexit.register(_stop_docs_server)


def _query(query):
    endpoint = _ensure_docs_server()
    request = Request(
        endpoint,
        data=query.encode(),
        headers={'Content-Type': 'application/sparql-query'},
        method='POST',
    )
    try:
        with urlopen(request, timeout=10) as response:
            return response.headers.get_content_type(), response.read().decode()
    except OSError:
        _stop_docs_server()
        raise


def _result_text(content_type, body):
    if content_type == 'text/turtle':
        return 'turtle', body.strip()

    result = json.loads(body)
    if 'boolean' in result:
        return 'text', str(result['boolean']).lower()

    variables = result['head']['vars']
    rows = ['\t'.join(variables)]
    for binding in result['results']['bindings']:
        rows.append(
            '\t'.join(binding.get(name, {}).get('value', '') for name in variables)
        )
    return 'text', '\n'.join(rows)


def _human_date(value):
    if not value:
        return ''
    parsed = date.fromisoformat(value) if isinstance(value, str) else value
    return f'{parsed.day} {parsed.strftime("%B %Y")}'


def _adr_metadata(value_date, status):
    status_key = str(status).lower()
    kind = _ADR_STATUS_ADMONITION.get(status_key, 'note')
    parts = []

    if status:
        parts.append(
            _ADR_STATUS.get(
                status_key,
                str(status).replace('_', ' ').title(),
            )
        )
    if value_date:
        parts.append(f':material-calendar-clock: {_human_date(value_date)}')

    title = ' · '.join(parts)
    return f'!!! {kind} "{title}"\n'


def define_env(env):
    """Register documentation macros."""

    def source_data(path, repository_path, indent, title):
        if not path.is_file():
            raise ValueError(f'Documentation source does not exist: {path}')
        source = path.read_text().rstrip('\n')
        syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
        github_url = _github_url(
            repository_path,
            env.conf.get('repo_url') or REPO_URL,
        )
        heading = (
            f'{title}<span class="example-source-link" markdown>'
            f':fontawesome-brands-github: [`{path.name}`]({github_url})'
            f'</span>'
        )
        body = f'```{syntax}\n{source}\n```'
        return (
            f'!!! example "{heading}"\n\n'
            f'{_indent_block(body, indent + 4)}\n'
        )

    @env.macro
    def adr_metadata(date, status):
        return _adr_metadata(date, status)

    @env.macro
    def example_data(name, indent=0, title='Source'):
        path = _example_path(name)
        return source_data(
            path,
            Path('docs/examples') / name,
            indent,
            title,
        )

    @env.macro
    def result_data(name, indent=0, title='Result'):
        path = RESULTS_DIR / name
        return source_data(
            path,
            Path('docs/results') / name,
            indent,
            title,
        )

    @env.macro
    def client_data(name, indent=0, title='Configuration'):
        path = CLIENTS_DIR / name
        return source_data(
            path,
            Path('docs/reference/clients') / name,
            indent,
            title,
        )

    @env.macro
    def query_data(name, indent=0, title='SPARQL query'):
        path = _query_path(name)
        if not path.is_file():
            raise ValueError(f'Query file does not exist: {name}')
        source = path.read_text().rstrip('\n')
        github_url = _github_url(
            Path('docs/queries') / name,
            env.conf.get('repo_url') or REPO_URL,
        )
        heading = (
            f'{title}<span class="example-source-link" markdown>'
            f':fontawesome-brands-github: [`{path.name}`]({github_url})'
            '</span>'
        )
        body = f'```sparql\n{source}\n```'
        return (
            f'!!! example "{heading}"\n\n'
            f'{_indent_block(body, indent + 4)}\n'
        )

    @env.macro
    def live_query(name):
        path = _query_path(name)
        if not path.is_file():
            raise ValueError(f'Query example does not exist: {name}')
        query = path.read_text().rstrip('\n')
        content_type, result = _query(query)
        syntax, result = _result_text(content_type, result)
        github_url = _github_url(
            Path('docs/queries') / name,
            env.conf.get('repo_url') or REPO_URL,
        )
        heading = (
            'Live query<span class="example-source-link" markdown>'
            f':fontawesome-brands-github: [`{path.name}`]({github_url})'
            '</span>'
        )
        body = (
            f'```sparql\n{query}\n```\n\n'
            f'```{syntax} title="Result"\n{result}\n```'
        )
        return f'!!! example "{heading}"\n\n{_indent_block(body, 4)}\n'

    @env.macro
    def live_api_examples():
        endpoint = _ensure_docs_server()
        curl = _command('curl')
        query = 'ASK { ?subject ?predicate ?object }'
        examples = [
            (
                'GET',
                [
                    curl,
                    '--silent',
                    '--show-error',
                    '--fail-with-body',
                    '--get',
                    endpoint,
                    '--data-urlencode',
                    f'query={query}',
                ],
                (
                    f"curl --get '{DISPLAY_ENDPOINT}' \\\n"
                    f"  --data-urlencode 'query={query}'"
                ),
            ),
            (
                'POST query body',
                [
                    curl,
                    '--silent',
                    '--show-error',
                    '--fail-with-body',
                    endpoint,
                    '--header',
                    'Content-Type: application/sparql-query',
                    '--data-binary',
                    query,
                ],
                (
                    f"curl '{DISPLAY_ENDPOINT}' \\\n"
                    "  --header 'Content-Type: application/sparql-query' \\\n"
                    f"  --data-binary '{query}'"
                ),
            ),
            (
                'POST form',
                [
                    curl,
                    '--silent',
                    '--show-error',
                    '--fail-with-body',
                    endpoint,
                    '--data-urlencode',
                    f'query={query}',
                ],
                f"curl '{DISPLAY_ENDPOINT}' \\\n  --data-urlencode 'query={query}'",
            ),
        ]
        tabs = []
        for title, command, display in examples:
            output = json.dumps(json.loads(_run(command)), indent=2)
            body = (
                f'```console\n{display}\n```\n\n'
                f'```json title="Response"\n{output}\n```'
            )
            tabs.append(f'=== "{title}"\n\n{_indent_block(body, 4)}')
        return '\n\n'.join(tabs)

    @env.macro
    def live_library_examples():
        endpoint = _ensure_docs_server()
        examples = [
            (
                ':simple-python: Python',
                'python.py',
                [sys.executable, str(LIBRARY_EXAMPLES_DIR / 'python.py'), endpoint],
                'python',
            ),
            (
                ':simple-javascript: JavaScript',
                'javascript.mjs',
                [
                    _command('node'),
                    str(LIBRARY_EXAMPLES_DIR / 'javascript.mjs'),
                    endpoint,
                ],
                'javascript',
            ),
        ]
        tabs = []
        for title, name, command, syntax in examples:
            path = LIBRARY_EXAMPLES_DIR / name
            output = _run(command, expected='CONSTRUCT: Alpha Centauri')
            source = path.read_text().rstrip('\n')
            body = (
                f'```{syntax}\n{source}\n```\n\n'
                f'```text title="Live result"\n{output}\n```'
            )
            tabs.append(f'=== "{title}"\n\n{_indent_block(body, 4)}')
        return '\n\n'.join(tabs)

    @env.macro
    def verify_clients():
        endpoint = _ensure_docs_server()
        query_files = {
            'select': QUERIES_DIR / 'names.rq',
            'ask': QUERIES_DIR / 'ask-data.rq',
            'construct': QUERIES_DIR / 'construct-name.rq',
        }
        clients = {
            'sq': (
                _command('sq'),
                [
                    (['-e', endpoint, 'graphs'], 'sparqld:alpha-centauri.yamlld'),
                    (['-e', endpoint, '-f', str(query_files['ask'])], 'true'),
                    (
                        ['-e', endpoint, '-f', str(query_files['construct'])],
                        'Alpha Centauri',
                    ),
                ],
            ),
            'rsparql': (
                _command(
                    'rsparql',
                    next(
                        iter(
                            sorted(
                                (ROOT_DIR / '.tools').glob(
                                    'apache-jena-*/bin/rsparql'
                                )
                            )
                        ),
                        None,
                    ),
                ),
                [
                    (
                        ['--service', endpoint, '--query', str(query_files['select'])],
                        'Alpha Centauri',
                    ),
                    (
                        ['--service', endpoint, '--query', str(query_files['ask'])],
                        'Yes',
                    ),
                    (
                        [
                            '--service',
                            endpoint,
                            '--query',
                            str(query_files['construct']),
                        ],
                        'Alpha Centauri',
                    ),
                ],
            ),
            'comunica-sparql': (
                _command(
                    'comunica-sparql',
                    ROOT_DIR / 'node_modules' / '.bin' / 'comunica-sparql',
                ),
                [
                    (
                        [f'sparql@{endpoint}', '-f', str(query_files['select'])],
                        'Alpha Centauri',
                    ),
                    ([f'sparql@{endpoint}', '-f', str(query_files['ask'])], 'true'),
                    (
                        [f'sparql@{endpoint}', '-f', str(query_files['construct'])],
                        'Alpha Centauri',
                    ),
                ],
            ),
            'sparqlquery': (
                _command('sparqlquery'),
                [
                    (
                        [endpoint, '--queryfile', str(query_files['select'])],
                        'Alpha Centauri',
                    ),
                    ([endpoint, '--queryfile', str(query_files['ask'])], 'true'),
                    (
                        [endpoint, '--queryfile', str(query_files['construct'])],
                        'Alpha Centauri',
                    ),
                ],
            ),
        }
        for _, (executable, checks) in clients.items():
            for arguments, expected in checks:
                _run([executable, *arguments], expected=expected)

        sq = clients['sq'][0]
        config = (CLIENTS_DIR / 'sq' / 'sq.toml').read_text().replace(
            DISPLAY_ENDPOINT,
            endpoint,
        )
        with tempfile.TemporaryDirectory(prefix='sparqld-docs-sq-') as directory:
            config_directory = Path(directory)
            (config_directory / '.sq.toml').write_text(config)
            _run(
                [sq, 'graphs'],
                expected='data:alpha-centauri.yamlld',
                cwd=directory,
            )
            _run(
                [sq, '-f', str(QUERIES_DIR / 'sq-named-graph.rq')],
                expected='Alpha Centauri',
                cwd=directory,
            )
        return (
            '<!-- sq, rsparql, Comunica, and sparqlquery passed '
            'live compatibility checks. -->'
        )

    @env.macro
    def command(value, indent=0):
        body = f'```console\n{value}\n```'
        return (
            '!!! command "Command"\n\n'
            f'{_indent_block(body, indent + 4)}\n'
        )

    @env.macro
    def directory_tree(directory):
        root = (ROOT_DIR / directory).resolve()
        if not root.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid directory path: {directory}')
        if not root.is_dir():
            raise ValueError(f'Directory does not exist: {directory}')

        repo_url = env.conf.get('repo_url') or REPO_URL
        root_relative = root.relative_to(ROOT_DIR)
        root_url = _github_url(root_relative, repo_url, directory=True)
        lines = [
            f':material-folder: **[`{root.name}/`]({root_url})**  ',
        ]

        def append_directory(directory, depth):
            entries = sorted(
                directory.iterdir(),
                key=lambda entry: (not entry.is_dir(), entry.name.lower()),
            )
            for index, entry in enumerate(entries):
                last = index == len(entries) - 1
                connector = '└──' if last else '├──'
                relative = entry.relative_to(ROOT_DIR)
                padding = '&nbsp;&nbsp;&nbsp;&nbsp;' * depth
                if entry.is_dir():
                    url = _github_url(relative, repo_url, directory=True)
                    lines.append(
                        f'{padding}{connector} :material-folder: '
                        f'**[`{entry.name}/`]({url})**  '
                    )
                    append_directory(entry, depth + 1)
                else:
                    url = _github_url(relative, repo_url)
                    icon = _EXAMPLE_ICONS.get(
                        entry.suffix.lower(), ':material-file-outline:'
                    )
                    lines.append(
                        f'{padding}{connector} {icon} [`{entry.name}`]({url})  '
                    )

        append_directory(root, 0)
        return '\n'.join(lines)


def on_post_build(env):
    """Stop the endpoint used to render live documentation examples."""
    _stop_docs_server()
