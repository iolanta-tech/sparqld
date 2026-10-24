"""MkDocs macros for project documentation."""

from __future__ import annotations

import json
import os
import subprocess
import textwrap
from datetime import date
from pathlib import Path
from types import MappingProxyType
from typing import Any

from pydantic import BaseModel, ConfigDict

from mkdocs_macros_sparqld import run_query, stop_server
from mkdocs_macros_sparqld.render import SelectResult, Term

ROOT_DIR = Path(__file__).resolve().parent
DOCS_DIR = ROOT_DIR / 'docs'
EXAMPLES_DIR = DOCS_DIR / 'examples'
CLIENTS_DIR = DOCS_DIR / 'reference' / 'clients'
CONVERSATIONS_DIR = DOCS_DIR / 'conversations'
QUERIES_DIR = DOCS_DIR / 'queries'
RESULTS_DIR = DOCS_DIR / 'results'
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
        '.yaml': 'yaml',
        '.yamlld': 'yaml',
        '.yml': 'yaml',
    }
)


def _example_path(name: str) -> Path:
    path = (EXAMPLES_DIR / name).resolve()
    if not path.is_relative_to(EXAMPLES_DIR.resolve()):
        raise ValueError(f'Invalid example path: {name}')
    return path


def _github_url(
    relative: str | Path,
    repo_url: str,
    directory: bool = False,
) -> str:
    relative = Path(relative).as_posix()
    operation = 'tree' if directory else 'blob'
    return f'{repo_url.rstrip("/")}/{operation}/main/{relative}'


def _repository_url(env: Any) -> str:
    return env.conf.get('repo_url') or REPO_URL


def _indent_block(body: str, indent: int) -> str:
    return textwrap.indent(body, ' ' * indent, lambda _: True)


def _command_text(command: list[str]) -> str:
    return ' '.join(map(str, command))


def _run(
    command: str | list[str],
    expected: str | None = None,
    environment: dict[str, str] | None = None,
    cwd: Path = ROOT_DIR,
    shell: bool = False,
) -> str:
    if isinstance(command, str):
        argv = ['/bin/sh', '-c', command]
    else:
        argv = command
    command_text = _command_text(argv)
    try:
        completed_process = subprocess.run(
            argv,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd,
            env=environment,
        )
    except FileNotFoundError as error:
        stop_server()
        raise RuntimeError(
            f'Required documentation command `{argv[0]}` was not found.'
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
            f'Documentation command did not return `{expected}`:\n{command_text}\n{output}',
        )
    return output


def _human_date(date_value: str | date | None) -> str:
    if not date_value:
        return ''
    parsed = (
        date.fromisoformat(date_value) if isinstance(date_value, str) else date_value
    )
    return f'{parsed.day} {parsed.strftime("%B %Y")}'


def adr_metadata(value_date: str | date | None, status: str) -> str:
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


def universe_metadata(adr_path: str, status: str) -> str:
    """Render the relationship and status of a supporting alternative page."""
    status_key = str(status).lower()
    unknown_status = ('note', str(status).title())
    kind, label = _UNIVERSE_STATUS.get(status_key, unknown_status)
    article = label.lower()
    return f'!!! {kind} "{label} alternative universe"\n\n    This is the {article} alternative universe from the [parent ADR]({adr_path}).\n'


def example_code(name: str, indent: int = 0) -> str:
    path = _example_path(name)
    source = path.read_text().rstrip('\n')
    syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    return _indent_block(f'```{syntax}\n{source}\n```\n', indent)


class Decision(BaseModel):
    """One row projected by the decision-log SELECT query."""

    model_config = ConfigDict(extra='ignore')

    graph: Term
    title: Term
    status: Term
    date: Term
    author: Term


Decision.model_rebuild(_types_namespace={'Term': Term})


def _decisions(body: str) -> list[Decision]:
    selected = SelectResult.model_validate_json(body)
    return [Decision.model_validate(binding) for binding in selected.solutions.bindings]


def _decision_heading(decision: Decision) -> str:
    title = decision.title.lexical_form
    page = decision.graph.lexical_form.rsplit('/', 1)[-1]
    return f'-   __[{title}]({page})__'


def _decision_card(decision: Decision) -> list[str]:
    status_key = decision.status.lexical_form.lower()
    status = _ADR_STATUS.get(
        status_key,
        status_key.replace('_', ' ').title(),
    )
    created = _human_date(decision.date.lexical_form)
    return [
        _decision_heading(decision),
        '',
        '    ---',
        '',
        f'    <span class="adr-status adr-status--{status_key}">{status}</span>',
        '',
        f'    :material-calendar-outline: <span class="adr-date">{created}</span>',
        '',
    ]


def decision_log():
    query = (QUERIES_DIR / 'decisions.rq').read_text()
    _, body = run_query(query)
    cards = ['<div class="grid cards adr-cards" markdown>', '']
    for decision in _decisions(body):
        cards.extend(_decision_card(decision))
    cards.append('</div>')
    return '\n'.join(cards)


def shell(
    command: str,
    env: dict[str, str] | None = None,
    output_format: str | None = None,
    indent: int = 0,
) -> str:
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
    rendered = f'```console\n{command}\n```\n\n```{json.__name__} title="Response"\n{output}\n```'
    return _indent_block(rendered, indent)


def client_compatibility():
    """Render the recorded SPARQL client checks."""
    if not CLIENT_COMPATIBILITY_PATH.is_file():
        raise RuntimeError(
            'Client compatibility record is missing. Run `j verify-clients` before building the documentation.',
        )
    for line in CLIENT_COMPATIBILITY_PATH.read_text().splitlines():
        if line.startswith('summary:'):
            summary = line.split(':', 1)[1].strip().strip('"')
            return f'<!-- {summary} -->\n'
    raise RuntimeError(f'{CLIENT_COMPATIBILITY_PATH} does not include a summary.')


def command(command_text: str, indent: int = 0) -> str:
    body = f'```console\n{command_text}\n```'
    return f'!!! command "Command"\n\n{_indent_block(body, indent + 4)}\n'


def _source_heading(
    path: Path,
    repository_path: str | Path,
    title: str,
    repo_url: str,
) -> str:
    github_url = _github_url(repository_path, repo_url)
    name = path.name
    return f'{title}<span class="example-source-link" markdown>:fontawesome-brands-github: [`{name}`]({github_url})</span>'


def _source_data(
    path: Path,
    repository_path: str | Path,
    indent: int,
    title: str,
    repo_url: str,
) -> str:
    if not path.is_file():
        raise ValueError(f'Documentation source does not exist: {path}')
    source = path.read_text().rstrip('\n')
    syntax = _EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    heading = _source_heading(path, repository_path, title, repo_url)
    body = f'```{syntax}\n{source}\n```'
    rendered = f'!!! example "{heading}"\n\n{_indent_block(body, 4)}\n'
    return _indent_block(rendered, indent)


def _directory_entries(directory: Path) -> list[Path]:
    return sorted(
        directory.iterdir(),
        key=lambda entry: (not entry.is_dir(), entry.name.lower()),
    )


def _tree_connector(index: int, count: int) -> str:
    if index == count - 1:
        return '└──'
    return '├──'


def _folder_line(
    entry: Path,
    relative: Path,
    connector: str,
    padding: str,
    repo_url: str,
) -> str:
    url = _github_url(relative, repo_url, directory=True)
    name = entry.name
    return f'{padding}{connector} :material-folder: **[`{name}/`]({url})**  '


def _file_line(
    entry: Path,
    relative: Path,
    connector: str,
    padding: str,
    repo_url: str,
) -> str:
    url = _github_url(relative, repo_url)
    icon = _EXAMPLE_ICONS.get(entry.suffix.lower(), ':material-file-outline:')
    name = entry.name
    return f'{padding}{connector} {icon} [`{name}`]({url})  '


def _append_entry(
    lines: list[str],
    entry: Path,
    connector: str,
    depth: int,
    repo_url: str,
) -> None:
    relative = entry.relative_to(ROOT_DIR)
    padding = '&nbsp;&nbsp;&nbsp;&nbsp;' * depth
    if entry.is_dir():
        lines.append(_folder_line(entry, relative, connector, padding, repo_url))
        _append_directory(lines, entry, depth + 1, repo_url)
        return
    lines.append(_file_line(entry, relative, connector, padding, repo_url))


def _append_directory(
    lines: list[str],
    directory: Path,
    depth: int,
    repo_url: str,
) -> None:
    entries = _directory_entries(directory)
    for index, entry in enumerate(entries):
        connector = _tree_connector(index, len(entries))
        _append_entry(lines, entry, connector, depth, repo_url)


def _conversation_paths(name: str) -> tuple[Path, Path]:
    directory = CONVERSATIONS_DIR / name
    prompt_path = directory / 'user.prompt'
    response_path = directory / 'codex.response'
    if not prompt_path.is_file() or not response_path.is_file():
        raise ValueError(f'Agent conversation does not exist: {name}')
    return prompt_path, response_path


def _message_url(name: str, path: Path, repo_url: str) -> str:
    return _github_url(Path('docs/conversations') / name / path.name, repo_url)


_MESSAGE_HEADING = """
**{speaker}**<span class="agent-message__source" markdown>[`{filename}`]({url})</span>
""".strip()


def _message_heading(speaker: str, path: Path, url: str) -> str:
    return _MESSAGE_HEADING.format(
        speaker=speaker,
        filename=path.name,
        url=url,
    )


def _agent_message(
    speaker: str,
    admonition: str,
    path: Path,
    name: str,
    repo_url: str,
) -> str:
    heading = _message_heading(speaker, path, _message_url(name, path, repo_url))
    body = _indent_block(path.read_text().strip(), 4)
    return f'!!! {admonition} "{heading}"\n\n{body}\n'


class EnvironmentMacros:
    """Macros that read the repository URL from the MkDocs environment."""

    def __init__(self, env: Any) -> None:
        self._env = env

    def source(self, path: str | Path, indent: int = 0, title: str = 'Source') -> str:
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

    def example_data(self, name: str, indent: int = 0, title: str = 'Source') -> str:
        return self.source(Path('docs/examples') / name, indent=indent, title=title)

    def result_data(self, name: str, indent: int = 0, title: str = 'Result') -> str:
        path = RESULTS_DIR / name
        return _source_data(
            path,
            Path('docs/results') / name,
            indent,
            title,
            _repository_url(self._env),
        )

    def client_data(
        self,
        name: str,
        indent: int = 0,
        title: str = 'Configuration',
    ) -> str:
        path = CLIENTS_DIR / name
        return _source_data(
            path,
            Path('docs/reference/clients') / name,
            indent,
            title,
            _repository_url(self._env),
        )

    def agent_conversation(self, name: str) -> str:
        prompt_path, response_path = _conversation_paths(name)
        repo_url = _repository_url(self._env)
        prompt = _agent_message('You', 'agent-user', prompt_path, name, repo_url)
        response = _agent_message(
            'Codex',
            'agent-codex',
            response_path,
            name,
            repo_url,
        )
        return f'{prompt}\n{response}'

    def directory_tree(self, directory: str | Path) -> str:
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


def _register_source_macros(env: Any, macros: EnvironmentMacros) -> None:
    env.macro(macros.source)
    env.macro(macros.example_data)
    env.macro(example_code)
    env.macro(macros.result_data)
    env.macro(macros.client_data)


def _register_page_macros(env: Any, macros: EnvironmentMacros) -> None:
    env.macro(adr_metadata)
    env.macro(universe_metadata)
    env.macro(decision_log)
    env.macro(shell)
    env.macro(client_compatibility)
    env.macro(command)
    env.macro(macros.agent_conversation)
    env.macro(macros.directory_tree)


def define_env(env: Any) -> None:
    """Register documentation macros."""
    macros = EnvironmentMacros(env)
    _register_source_macros(env, macros)
    _register_page_macros(env, macros)
