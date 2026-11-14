"""Typed roadmap nodes and deterministic Mermaid declarations."""

from dataclasses import dataclass
from hashlib import sha256
from typing import ClassVar, Literal


@dataclass(frozen=True)
class RoadmapNode:
    iri: str
    title: str
    status: str | None = None
    group: str | None = None
    group_title: str | None = None
    mermaid_shape: ClassVar[str]
    default_status: ClassVar[str] = 'unrecorded'

    def declaration(self) -> str:
        label = _escape_label(self.title)
        return f'    {_identifier(self.iri)}{self.mermaid_shape.format(label=label)}'

    def status_class(self) -> str:
        status = self.status or self.default_status
        return f'status_{_safe_status(status)}'

    def styled_nodes(self) -> tuple['RoadmapNode', ...]:
        return (self,)


@dataclass(frozen=True)
class Goal(RoadmapNode):
    kind: Literal['goal'] = 'goal'
    mermaid_shape: ClassVar[str] = '("{label}")'
    default_status: ClassVar[str] = 'open'


@dataclass(frozen=True)
class Bug(RoadmapNode):
    kind: Literal['bug'] = 'bug'
    mermaid_shape: ClassVar[str] = '("{label}")'
    default_status: ClassVar[str] = 'open'

    def status_class(self) -> str:
        if not self.status or self.status.lower() == 'open':
            return 'status_open_bug'
        return super().status_class()


@dataclass(frozen=True)
class Task(RoadmapNode):
    kind: Literal['task'] = 'task'
    mermaid_shape: ClassVar[str] = '("{label}")'
    default_status: ClassVar[str] = 'open'


@dataclass(frozen=True)
class PrerequisiteState(RoadmapNode):
    kind: Literal['state'] = 'state'
    mermaid_shape: ClassVar[str] = '(["{label}"])'


@dataclass(frozen=True)
class Position(RoadmapNode):
    kind: Literal['position'] = 'position'
    mermaid_shape: ClassVar[str] = '["{label}"]'


@dataclass(frozen=True)
class Decision:
    iri: str
    title: str
    positions: tuple[Position, ...]

    def styled_nodes(self) -> tuple[RoadmapNode, ...]:
        return self.positions

    def declaration(self) -> str:
        lines = [
            f'    subgraph {_identifier(self.iri)}["{_escape_label(self.title)}"]',
            '        direction TB',
        ]
        lines.extend(f'        {position.declaration().strip()}' for position in self.positions)
        lines.extend(
            f'        {_identifier(first.iri)} ~~~ {_identifier(second.iri)}'
            for first, second in zip(self.positions, self.positions[1:])
        )
        return '\n'.join((*lines, '    end'))


@dataclass(frozen=True)
class Roadmap:
    nodes: tuple[RoadmapNode | Decision, ...]
    edges: frozenset[tuple[str, str]]

    def render(self) -> str:
        declarations = tuple(node.declaration() for node in self.nodes)
        lines = ('```mermaid', 'flowchart LR', *declarations)
        edges = tuple(
            f'    {_identifier(source)} --> {_identifier(target)}'
            for source, target in sorted(self.edges)
        )
        styled_nodes = tuple(member for node in self.nodes for member in node.styled_nodes())
        statuses = sorted({node.status_class() for node in styled_nodes})
        colors = {'undecided': '#fff2cc', 'accepted': '#d9ead3', 'rejected': '#f4cccc', 'open': '#d9eaf7', 'open_bug': '#f4cccc', 'closed': '#d9ead3', 'cancelled': '#f4cccc', 'unrecorded': '#eeeeee'}
        styles = tuple(
            f'    classDef {name} fill:{colors.get(name.removeprefix("status_"), "#eeeeee")},color:#222,stroke:#888'
            for name in statuses
        )
        assignments = tuple(f'    class {_identifier(node.iri)} {node.status_class()}' for node in styled_nodes)
        return '\n'.join((*lines, *edges, *styles, *assignments, '```'))


def _safe_status(status: str | None) -> str:
    normalized = status.lower() if status else 'unrecorded'
    return normalized if normalized in {'undecided', 'accepted', 'rejected', 'open', 'closed', 'cancelled'} else 'unrecorded'


def _identifier(iri: str) -> str:
    digest = sha256(iri.encode()).hexdigest()
    return f'node_{digest}'


def _escape_label(text: str) -> str:
    return ''.join(
        character if character.isalnum() or character == ' ' else f'#{ord(character)};'
        for character in ' '.join(text.split())
    )
