"""MkDocs macros for project documentation."""

import json
import os
import subprocess
import textwrap
from datetime import date
from pathlib import Path
from types import MappingProxyType

from mkdocs_macros_sparqld import run_query, stop_server

ROOT_DIR = Path(__file__).resolve().parent
DOCS_DIR = ROOT_DIR / 'docs'
EXAMPLES_DIR = DOCS_DIR / 'examples'
CLIENTS_DIR = ROOT_DIR / 'docs' / 'reference' / 'clients'
CONVERSATIONS_DIR = ROOT_DIR / 'docs' / 'conversations'
QUERIES_DIR = ROOT_DIR / 'docs' / 'queries'
RESULTS_DIR = ROOT_DIR / 'docs' / 'results'
REPO_URL = 'https://github.com/iolanta-tech/sparqld'
CLIENT_COMPATIBILITY_PATH = RESULTS_DIR / 'client-compatibility.yamlld'

# @todo(generated-roadmap): Render the roadmap from the sparqld dataset.
# description: >
#   Configure the documentation build with `extra.sparqld.endpoint` pointing to
#   an already running sparqld server over the repository root. That server
#   loads native documents and Riddles contributions; the pluglet only queries
#   it and does not own its process lifecycle.
#
#   Add docs/project/mission.yamlld declaring `sparqld:mission`, the enduring
#   project mission, and any roadmap outcome that is not owned by a code
#   annotation. Add docs/queries/roadmap.rq, a SELECT query returning each
#   roadmap item, title, and optional urn:riddle:blockedBy item. Add
#   `roadmap_mermaid('docs/queries/roadmap.rq')`; it
#   runs that stored query and returns a deterministic `flowchart LR` block.
# rendering:
#   - Give each IRI a deterministic Mermaid-safe node ID and HTML-escape title
#     text before inserting it into a node label.
#   - For `item urn:riddle:blockedBy blocker`, emit `blocker --> item`; dependencies
#     therefore appear left of the mission without reversing RDF semantics.
#   - Replace the handwritten graph in docs/project/roadmap.md with the macro;
#     it must consume SPARQL results, never source comments directly.
# acceptance:
#   - Unit-test macro rendering for a graph with two blockers, a transitive
#     blocker, titles requiring escaping, and a mission node.
#   - Build the documentation against an externally managed endpoint.
#   - Validate the rendered roadmap in Chromium through Playwright after the
#     documentation change.
# blocked-by:
#   - riddles-integration
#   - existing-sparqld-endpoint

_ADR_STATUS = MappingProxyType(
    {
        'draft': 'Draft',
        'undecided': 'Undecided',
        'decided': 'Decided',
    }
)


_ADR_STATUS_ADMONITION = MappingProxyType(
    {
        'draft': 'note',
        'undecided': 'warning',
        'decided': 'success',
    }
)


_EXAMPLE_ICONS = MappingProxyType(
    {
        '.json': ':material-code-json:',
        '.jsonld': ':material-code-json:',
        '.md': ':material-language-markdown:',
        '.rq': ':material-database-search-outline:',
        '.sh': ':material-console:',
        '.yaml': ':simple-yaml:',
        '.yamlld': ':simple-yaml:',
        '.yml': ':simple-yaml:',
    }
)


_EXAMPLE_SYNTAXES = MappingProxyType(
    {
        '.json': 'json',
        '.jsonld': 'json',
        '.md': 'markdown',
        '.mjs': 'javascript',
        '.py': 'python',
        '.rq': 'sparql',
        '.sh': 'console',
        '.toml': 'toml',
        '.tsv': 'text',
        '.txt': 'text',
        '.yaml': 'yaml',
        '.yamlld': 'yaml',
        '.yml': 'yaml',
    }
)


def _example_path(name):
    path = (EXAMPLES_DIR / name).resolve()
    if not path.is_relative_to(EXAMPLES_DIR.resolve()):
        raise ValueError(f'Invalid example path: {name}')
    return path


def _github_url(relative, repo_url, directory=False):
    relative = Path(relative).as_posix()
    operation = 'tree' if directory else 'blob'
    return f'{repo_url.rstrip("/")}/{operation}/main/{relative}'


def _repository_url(env):
    return env.conf.get('repo_url') or REPO_URL


def _indent_block(body, indent):
    return textwrap.indent(body, ' ' * indent, lambda _: True)


def _command_text(command):
    return ' '.join(map(str, command))


def _run(command, expected=None, environment=None, cwd=ROOT_DIR, shell=False):
    if shell:
        command = ['/bin/sh', '-c', command]
    command_text = _command_text(command)
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
            f'Documentation command failed: {command_text}\n{detail}'
        ) from error
    output = completed_process.stdout.strip()
    if expected and expected not in output:
        stop_server()
        raise RuntimeError(
            f'Documentation command did not return `{expected}`:\n'
            f'{command_text}\n{output}'
        )
    return output


def _human_date(date_value):
    if not date_value:
        return ''
    parsed = (
        date.fromisoformat(date_value) if isinstance(date_value, str) else date_value
    )
    return f'{parsed.day} {parsed.strftime("%B %Y")}'


def adr_metadata(value_date, status):
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


_UNIVERSE_STATUS = MappingProxyType(
    {
        'chosen': ('success', 'Chosen'),
        'excluded': ('failure', 'Excluded'),
        'undecided': ('warning', 'Undecided'),
    }
)


def universe_metadata(adr_path, status):
    """Render the relationship and status of a supporting alternative page."""
    status_key = str(status).lower()
    unknown_status = ('note', str(status).title())
    kind, label = _UNIVERSE_STATUS.get(status_key, unknown_status)
    article = label.lower()
    return (
        f'!!! {kind} "{label} alternative universe"\n\n'
        f'    This is the {article} alternative universe from the '
        f'[parent ADR]({adr_path}).\n'
    )


def example_code(name, indent=0):
    path = _example_path(name)
    source = path.read_text().rstrip('\n')
    syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    return _indent_block(f'```{syntax}\n{source}\n```\n', indent)


def decision_log():
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


def shell(command, env=None, output_format=None, indent=0):
    """Run a shell command and render its JSON output."""
    if output_format is None:
        output_format = json.__name__
    if output_format != json.__name__:
        raise ValueError(f'Unsupported shell output format: {output_format}')
    try:
        output = json.dumps(
            json.loads(
                _run(
                    command,
                    environment={
                        **os.environ,
                        **{
                            name: str(env_value)
                            for name, env_value in (env or {}).items()
                        },
                    },
                    shell=True,
                )
            ),
            indent=2,
        )
    except json.JSONDecodeError as error:
        stop_server()
        raise RuntimeError(
            f'Documentation shell command did not return JSON: {command}'
        ) from error
    return _indent_block(
        f'```console\n{command}\n```\n\n'
        f'```{json.__name__} title="Response"\n{output}\n```',
        indent,
    )


def client_compatibility():
    """Render the recorded SPARQL client checks."""
    if not CLIENT_COMPATIBILITY_PATH.is_file():
        raise RuntimeError(
            'Client compatibility record is missing. '
            'Run `j verify-clients` before building the documentation.'
        )
    for line in CLIENT_COMPATIBILITY_PATH.read_text().splitlines():
        if line.startswith('summary:'):
            summary = line.split(':', 1)[1].strip().strip('"')
            return f'<!-- {summary} -->\n'
    raise RuntimeError(f'{CLIENT_COMPATIBILITY_PATH} does not include a summary.')


def command(command_text, indent=0):
    body = f'```console\n{command_text}\n```'
    return f'!!! command "Command"\n\n{_indent_block(body, indent + 4)}\n'


def _source_data(path, repository_path, indent, title, repo_url):
    if not path.is_file():
        raise ValueError(f'Documentation source does not exist: {path}')
    source = path.read_text().rstrip('\n')
    syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    github_url = _github_url(
        repository_path,
        repo_url,
    )
    heading = (
        f'{title}<span class="example-source-link" markdown>'
        f':fontawesome-brands-github: [`{path.name}`]({github_url})'
        f'</span>'
    )
    body = f'```{syntax}\n{source}\n```'
    rendered = f'!!! example "{heading}"\n\n{_indent_block(body, 4)}\n'
    return _indent_block(rendered, indent)


def _append_directory(lines, directory, depth, repo_url):
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
                f'{padding}{connector} :material-folder: **[`{entry.name}/`]({url})**  '
            )
            _append_directory(lines, entry, depth + 1, repo_url)
        else:
            url = _github_url(relative, repo_url)
            icon = _EXAMPLE_ICONS.get(entry.suffix.lower(), ':material-file-outline:')
            lines.append(f'{padding}{connector} {icon} [`{entry.name}`]({url})  ')


class EnvironmentMacros:
    """Macros that read the repository URL from the MkDocs environment."""

    def __init__(self, env):
        self._env = env

    def source(self, path, indent=0, title='Source'):
        """Render any project file as an example admonition."""
        file_path = (ROOT_DIR / path).resolve()
        if not file_path.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid path outside the project: {path}')
        return _source_data(
            file_path,
            Path(path),
            indent,
            title,
            _repository_url(self._env),
        )

    def example_data(self, name, indent=0, title='Source'):
        return self.source(Path('docs/examples') / name, indent=indent, title=title)

    def result_data(self, name, indent=0, title='Result'):
        path = RESULTS_DIR / name
        return _source_data(
            path,
            Path('docs/results') / name,
            indent,
            title,
            _repository_url(self._env),
        )

    def client_data(self, name, indent=0, title='Configuration'):
        path = CLIENTS_DIR / name
        return _source_data(
            path,
            Path('docs/reference/clients') / name,
            indent,
            title,
            _repository_url(self._env),
        )

    def agent_conversation(self, name):
        directory = CONVERSATIONS_DIR / name
        prompt_path = directory / 'user.prompt'
        response_path = directory / 'codex.response'
        if not prompt_path.is_file() or not response_path.is_file():
            raise ValueError(f'Agent conversation does not exist: {name}')

        repo_url = _repository_url(self._env)
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

    def directory_tree(self, directory):
        root = (ROOT_DIR / directory).resolve()
        if not root.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid directory path: {directory}')
        if not root.is_dir():
            raise ValueError(f'Directory does not exist: {directory}')

        repo_url = _repository_url(self._env)
        root_relative = root.relative_to(ROOT_DIR)
        root_url = _github_url(root_relative, repo_url, directory=True)
        lines = [
            f':material-folder: **[`{root.name}/`]({root_url})**  ',
        ]

        _append_directory(lines, root, 0, repo_url)
        return '\n'.join(lines)


def define_env(env):
    """Register documentation macros."""
    macros = EnvironmentMacros(env)
    env.macro(adr_metadata)
    env.macro(universe_metadata)
    env.macro(macros.source)
    env.macro(macros.example_data)
    env.macro(example_code)
    env.macro(macros.result_data)
    env.macro(macros.client_data)
    env.macro(macros.agent_conversation)
    env.macro(decision_log)
    env.macro(shell)
    env.macro(client_compatibility)
    env.macro(command)
    env.macro(macros.directory_tree)
