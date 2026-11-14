"""Refresh the roadmap projection of GitHub issues and their blockers."""

import subprocess
import tempfile
from pathlib import Path

from pydantic import BaseModel, TypeAdapter
from ruamel.yaml import YAML
from ruamel.yaml.emitter import RoundTripEmitter
from ruamel.yaml.error import YAMLError
from ruamel.yaml.scalarstring import DoubleQuotedScalarString

ROOT = Path(__file__).resolve().parents[2]
DESTINATION_DIR = ROOT / 'docs/project/roadmap/github-issues'
LEGACY_DESTINATION = ROOT / 'docs/project/roadmap/github-issues.yamlld'
REPOSITORY = 'iolanta-tech/sparqld'
MISSION = 'urn:sparqld:roadmap:mission'


class DoubleQuoteEmitter(RoundTripEmitter):
    """Use double quotes when YAML requires a quoted scalar."""

    def choose_scalar_style(self) -> str:
        style = super().choose_scalar_style()
        return '"' if style == "'" else style


class Label(BaseModel):
    """A GitHub issue label."""

    name: str


class Issue(BaseModel):
    """GitHub fields used by the roadmap."""

    number: int
    html_url: str
    title: str
    body: str | None = None
    state: str
    state_reason: str | None = None
    pull_request: dict | None = None
    labels: list[Label] = []


def issue_types(issue: Issue) -> str | list[str]:
    """Represent GitHub work as tasks and retain bug classification."""
    task_type = 'pm:Task'
    if any(label.name.casefold() == 'bug' for label in issue.labels):
        return [task_type, 'urn:sparqld:vocabulary:Bug']
    return task_type


def has_type(record: dict, expected: str) -> bool:
    """Recognize generated records with scalar or multiple RDF types."""
    types = record.get('$type')
    return types == expected or (
        isinstance(types, list) and expected in types
    )


def fetch(endpoint: str) -> list[Issue]:
    """Read every page before returning any records."""
    result = subprocess.run(
        ['gh', 'api', endpoint, '--paginate', '--slurp'],
        check=True, capture_output=True, text=True, cwd=ROOT,
    )
    pages = TypeAdapter(list[list[Issue]]).validate_json(result.stdout)
    return [item for page in pages for item in page]


def issue_status(issue: Issue) -> str:
    """Map GitHub states to roadmap status resources."""
    return 'cancelled' if issue.state_reason == 'not_planned' else issue.state


def issue_document(
    issue: Issue, blockers: list[Issue], *, blocks_mission: bool = True,
) -> dict:
    """Build an issue root with nested external dependencies."""
    record = {
        '$id': issue.html_url,
        '$type': issue_types(issue),
        'title': issue.title,
        'status': issue_status(issue),
    }
    if blocks_mission:
        record['blocks'] = MISSION
    if blockers:
        dependencies = []
        for item in sorted(blockers, key=lambda blocker: blocker.html_url):
            if item.html_url.startswith(f'https://github.com/{REPOSITORY}/issues/'):
                dependencies.append(item.html_url)
            else:
                dependencies.append({
                    '$id': item.html_url,
                    '$type': issue_types(item),
                    'title': item.title,
                    'status': issue_status(item),
                })
        record['blockedBy'] = dependencies[0] if len(dependencies) == 1 else dependencies
    return {
        'render_macros': False,
        DoubleQuotedScalarString('@context'): 'urn:sparqld:roadmap',
        **record,
    }


def mission_blockers(issues: list[Issue], dependencies: dict[str, list[Issue]]) -> set[str]:
    """Anchor each terminal dependency component to the visible mission graph."""
    local = {issue.html_url: issue for issue in issues}
    downstream = {url: set() for url in local}
    for blocked_url, blockers in dependencies.items():
        blocked = local[blocked_url]
        if blocked.state == 'closed' and blocked.state_reason != 'not_planned':
            continue
        for blocker in blockers:
            if blocker.html_url in local:
                downstream[blocker.html_url].add(blocked_url)

    reachable = {}
    for url in local:
        visited = {url}
        pending = [url]
        while pending:
            for target in downstream[pending.pop()] - visited:
                visited.add(target)
                pending.append(target)
        reachable[url] = visited

    anchors = set()
    for url, targets in reachable.items():
        component = {target for target in targets if url in reachable[target]}
        if targets == component:
            anchors.add(min(component))
    return anchors


def is_generated_issue_file(path: Path) -> bool:
    """Identify a prior issue record before deleting a stale generated file."""
    try:
        contents = path.read_text()
        if path.suffix == '.md':
            lines = contents.splitlines(keepends=True)
            if not lines or lines[0].strip() != '---':
                return False
            closing = next(
                (index for index, line in enumerate(lines[1:], 1) if line.strip() == '---'),
                None,
            )
            if closing is None:
                return False
            contents = ''.join(lines[1:closing])
        document = YAML(typ='safe').load(contents)
    except (OSError, KeyError, TypeError, ValueError, YAMLError):
        return False
    if not isinstance(document, dict):
        return False
    graph = document.get('$graph', [document])
    if not isinstance(graph, list):
        return False
    if len(graph) != 1 or not isinstance(graph[0], dict):
        return False
    record = graph[0]
    return (
        record.get('$id') == f'https://github.com/{REPOSITORY}/issues/{path.stem}'
        and (
            has_type(record, 'pm:Task')
            or has_type(record, 'urn:sparqld:vocabulary:GitHubIssue')
        )
    )


def is_generated_legacy_file() -> bool:
    """Check that the legacy path contains the former aggregate projection."""
    try:
        graph = YAML(typ='safe').load(LEGACY_DESTINATION.read_text())['$graph']
    except (OSError, KeyError, TypeError, ValueError, YAMLError):
        return False
    if not isinstance(graph, list) or not graph:
        return False
    return all(
        isinstance(record, dict)
        and has_type(record, 'urn:sparqld:vocabulary:GitHubIssue')
        and isinstance(record.get('$id'), str)
        and record['$id'].startswith(f'https://github.com/{REPOSITORY}/issues/')
        and record['$id'].rsplit('/', 1)[-1].isdecimal()
        for record in graph
    )


def main() -> None:
    """Fetch the complete projection, then replace generated issue files."""
    issues = [
        item for item in fetch(f'repos/{REPOSITORY}/issues?state=all&per_page=100')
        if item.pull_request is None
    ]
    dependencies = {}
    for issue in sorted(issues, key=lambda item: item.number):
        blockers = fetch(
            f'repos/{REPOSITORY}/issues/{issue.number}/dependencies/blocked_by?per_page=100'
        )
        dependencies[issue.html_url] = blockers
    anchors = mission_blockers(issues, dependencies)
    documents = [
        (issue, issue_document(
            issue, dependencies[issue.html_url], blocks_mission=issue.html_url in anchors,
        ))
        for issue in sorted(issues, key=lambda item: item.number)
    ]

    DESTINATION_DIR.mkdir(parents=True, exist_ok=True)
    yaml = YAML()
    yaml.Emitter = DoubleQuoteEmitter
    yaml.width = 100
    for issue, document in documents:
        for suffix in ('.md', '.yamlld'):
            destination = DESTINATION_DIR / f'{issue.number}{suffix}'
            if destination.exists() and not is_generated_issue_file(destination):
                raise ValueError(f'Refusing to overwrite non-generated file: {destination}')
    for issue, document in documents:
        destination = DESTINATION_DIR / f'{issue.number}.md'
        temporary = None
        try:
            with tempfile.NamedTemporaryFile(mode='w', dir=DESTINATION_DIR, delete=False) as stream:
                temporary = Path(stream.name)
                stream.write('---\n')
                yaml.dump(document, stream)
                stream.write('---\n\n')
                stream.write(issue.body or '')
            temporary.replace(destination)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)

    active_names = {f'{issue.number}.md' for issue, _ in documents}
    for path in (*DESTINATION_DIR.glob('*.yamlld'), *DESTINATION_DIR.glob('*.md')):
        if path.name not in active_names and path.stem.isdecimal() and is_generated_issue_file(path):
            path.unlink()
    if is_generated_legacy_file():
        LEGACY_DESTINATION.unlink()
    print(f'Synchronized {len(issues)} issues into {DESTINATION_DIR.relative_to(ROOT)}')


if __name__ == '__main__':
    main()
