"""MkDocs macros for project documentation."""

import json
import shutil
import subprocess
import sys
import tempfile
from datetime import date
from pathlib import Path

from mkdocs_macros_sparqld import ensure_endpoint, run_query, stop_server

ROOT_DIR = Path(__file__).resolve().parent
DOCS_DIR = ROOT_DIR / 'docs'
EXAMPLES_DIR = DOCS_DIR / 'examples'
CLIENTS_DIR = ROOT_DIR / 'docs' / 'reference' / 'clients'
CONVERSATIONS_DIR = ROOT_DIR / 'docs' / 'conversations'
LIBRARY_EXAMPLES_DIR = ROOT_DIR / 'docs' / 'reference' / 'libraries'
QUERIES_DIR = ROOT_DIR / 'docs' / 'queries'
RESULTS_DIR = ROOT_DIR / 'docs' / 'results'
REPO_URL = 'https://github.com/iolanta-tech/sparqld'
DISPLAY_ENDPOINT = 'http://127.0.0.1:7737/'


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
        completed_process = subprocess.run(
            command,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd,
            env=environment,
        )
    except FileNotFoundError as error:
        stop_server()
        raise RuntimeError(
            f'Required documentation command `{command[0]}` was not found.'
        ) from error
    except subprocess.CalledProcessError as error:
        stop_server()
        detail = (error.stderr or error.stdout).strip()
        raise RuntimeError(
            f'Documentation command failed: {" ".join(map(str, command))}\n{detail}'
        ) from error
    output = completed_process.stdout.strip()
    if expected and expected not in output:
        stop_server()
        raise RuntimeError(
            f'Documentation command did not return `{expected}`: '
            f'{" ".join(map(str, command))}\n{output}'
        )
    return output


def _human_date(date_value):
    if not date_value:
        return ''
    parsed = (
        date.fromisoformat(date_value) if isinstance(date_value, str) else date_value
    )
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


class DocumentationMacros:
    """Documentation macros bound to one MkDocs environment."""

    def __init__(self, env):
        self._env = env

    def _source_data(self, path, repository_path, indent, title):
        if not path.is_file():
            raise ValueError(f'Documentation source does not exist: {path}')
        source = path.read_text().rstrip('\n')
        syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
        github_url = _github_url(
            repository_path,
            self._env.conf.get('repo_url') or REPO_URL,
        )
        heading = (
            f'{title}<span class="example-source-link" markdown>'
            f':fontawesome-brands-github: [`{path.name}`]({github_url})'
            f'</span>'
        )
        body = f'```{syntax}\n{source}\n```'
        rendered = f'!!! example "{heading}"\n\n{_indent_block(body, 4)}\n'
        return _indent_block(rendered, indent)

    def adr_metadata(self, date, status):
        return _adr_metadata(date, status)

    def source(self, path, indent=0, title='Source'):
        """Render any project file as an example admonition."""
        file_path = (ROOT_DIR / path).resolve()
        if not file_path.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid path outside the project: {path}')
        return self._source_data(file_path, Path(path), indent, title)

    def example_data(self, name, indent=0, title='Source'):
        return self.source(Path('docs/examples') / name, indent=indent, title=title)

    def example_code(self, name, indent=0):
        path = _example_path(name)
        source = path.read_text().rstrip('\n')
        syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
        return _indent_block(f'```{syntax}\n{source}\n```\n', indent)

    def result_data(self, name, indent=0, title='Result'):
        path = RESULTS_DIR / name
        return self._source_data(
            path,
            Path('docs/results') / name,
            indent,
            title,
        )

    def client_data(self, name, indent=0, title='Configuration'):
        path = CLIENTS_DIR / name
        return self._source_data(
            path,
            Path('docs/reference/clients') / name,
            indent,
            title,
        )

    def agent_conversation(self, name):
        directory = CONVERSATIONS_DIR / name
        prompt_path = directory / 'user.prompt'
        response_path = directory / 'codex.response'
        if not prompt_path.is_file() or not response_path.is_file():
            raise ValueError(f'Agent conversation does not exist: {name}')

        repo_url = self._env.conf.get('repo_url') or REPO_URL
        prompt_url = _github_url(
            Path('docs/conversations') / name / prompt_path.name,
            repo_url,
        )
        response_url = _github_url(
            Path('docs/conversations') / name / response_path.name,
            repo_url,
        )
        prompt_heading = (
            '**You**'
            '<span class="agent-message__source" markdown>'
            f'[`{prompt_path.name}`]({prompt_url})'
            '</span>'
        )
        response_heading = (
            '**Codex**'
            '<span class="agent-message__source" markdown>'
            f'[`{response_path.name}`]({response_url})'
            '</span>'
        )
        prompt = _indent_block(prompt_path.read_text().strip(), 4)
        response = _indent_block(response_path.read_text().strip(), 4)
        return (
            f'!!! agent-user "{prompt_heading}"\n\n{prompt}\n\n'
            f'!!! agent-codex "{response_heading}"\n\n{response}\n'
        )

    def decision_log(self):
        query = (QUERIES_DIR / 'decisions.rq').read_text()
        _, body = run_query(query)
        bindings = json.loads(body)['results']['bindings']
        cards = ['<div class="grid cards adr-cards" markdown>', '']
        for binding in bindings:
            page = binding['graph']['value'].rsplit('/', 1)[-1]
            title = binding['title']['value']
            status_key = binding['status']['value'].lower()
            status = _ADR_STATUS.get(
                status_key,
                status_key.replace('_', ' ').title(),
            )
            date = _human_date(binding['date']['value'])
            cards.extend(
                [
                    f'-   __[{title}]({page})__',
                    '',
                    '    ---',
                    '',
                    (
                        f'    <span class="adr-status adr-status--{status_key}">'
                        f'{status}</span>'
                    ),
                    '',
                    (
                        f'    :material-calendar-outline: '
                        f'<span class="adr-date">{date}</span>'
                    ),
                    '',
                ]
            )
        cards.append('</div>')
        return '\n'.join(cards)

    def live_api_examples(self):
        endpoint = ensure_endpoint()
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
                f'```console\n{display}\n```\n\n```json title="Response"\n{output}\n```'
            )
            tabs.append(f'=== "{title}"\n\n{_indent_block(body, 4)}')
        return '\n\n'.join(tabs)

    def live_library_examples(self):
        endpoint = ensure_endpoint()
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

    def verify_clients(self):
        endpoint = ensure_endpoint()
        query_files = {
            'select': QUERIES_DIR / 'names.rq',
            'ask': QUERIES_DIR / 'ask-data.rq',
            'construct': QUERIES_DIR / 'construct-name.rq',
        }
        clients = {
            'sq': (
                _command('sq'),
                [
                    (
                        ['-e', endpoint, 'graphs'],
                        'sparqld:examples/alpha-centauri.yamlld',
                    ),
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
                                (ROOT_DIR / '.tools').glob('apache-jena-*/bin/rsparql')
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
        for executable, checks in clients.values():
            for arguments, expected in checks:
                _run([executable, *arguments], expected=expected)

        sq = clients['sq'][0]
        config = (
            (CLIENTS_DIR / 'sq' / 'sq.toml')
            .read_text()
            .replace(
                DISPLAY_ENDPOINT,
                endpoint,
            )
        )
        with tempfile.TemporaryDirectory(prefix='sparqld-docs-sq-') as directory:
            config_directory = Path(directory)
            (config_directory / '.sq.toml').write_text(config)
            _run(
                [sq, 'graphs'],
                expected='data:index.md',
                cwd=directory,
            )
            _run(
                [sq, '-f', str(QUERIES_DIR / 'sq-named-graph.rq')],
                expected='sparqld',
                cwd=directory,
            )
        return (
            '<!-- sq, rsparql, Comunica, and sparqlquery passed '
            'live compatibility checks. -->'
        )

    def command(self, command_text, indent=0):
        body = f'```console\n{command_text}\n```'
        return f'!!! command "Command"\n\n{_indent_block(body, indent + 4)}\n'

    def _append_directory(self, lines, directory, depth, repo_url):
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
                self._append_directory(lines, entry, depth + 1, repo_url)
            else:
                url = _github_url(relative, repo_url)
                icon = _EXAMPLE_ICONS.get(
                    entry.suffix.lower(), ':material-file-outline:'
                )
                lines.append(f'{padding}{connector} {icon} [`{entry.name}`]({url})  ')

    def directory_tree(self, directory):
        root = (ROOT_DIR / directory).resolve()
        if not root.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid directory path: {directory}')
        if not root.is_dir():
            raise ValueError(f'Directory does not exist: {directory}')

        repo_url = self._env.conf.get('repo_url') or REPO_URL
        root_relative = root.relative_to(ROOT_DIR)
        root_url = _github_url(root_relative, repo_url, directory=True)
        lines = [
            f':material-folder: **[`{root.name}/`]({root_url})**  ',
        ]

        self._append_directory(lines, root, 0, repo_url)
        return '\n'.join(lines)


def define_env(env):
    """Register documentation macros."""
    macros = DocumentationMacros(env)
    env.macro(macros.adr_metadata)
    env.macro(macros.source)
    env.macro(macros.example_data)
    env.macro(macros.example_code)
    env.macro(macros.result_data)
    env.macro(macros.client_data)
    env.macro(macros.agent_conversation)
    env.macro(macros.decision_log)
    env.macro(macros.live_api_examples)
    env.macro(macros.live_library_examples)
    env.macro(macros.verify_clients)
    env.macro(macros.command)
    env.macro(macros.directory_tree)
