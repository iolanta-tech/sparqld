"""Typed roadmap nodes and deterministic Mermaid declarations."""

from dataclasses import dataclass
from hashlib import sha256
from typing import ClassVar, Literal


@dataclass(frozen=True)
class RoadmapNode:
    iri: str
    title: str
    mermaid_shape: ClassVar[str]

    def declaration(self) -> str:
        label = _escape_label(self.title)
        return f'    {_identifier(self.iri)}{self.mermaid_shape.format(label=label)}'


@dataclass(frozen=True)
class Goal(RoadmapNode):
    kind: Literal['goal'] = 'goal'
    mermaid_shape: ClassVar[str] = '("{label}")'


@dataclass(frozen=True)
class Bug(RoadmapNode):
    kind: Literal['bug'] = 'bug'
    mermaid_shape: ClassVar[str] = '[{label}]'


@dataclass(frozen=True)
class PrerequisiteState(RoadmapNode):
    kind: Literal['state'] = 'state'
    mermaid_shape: ClassVar[str] = '(["{label}"])'


@dataclass(frozen=True)
class Roadmap:
    nodes: tuple[RoadmapNode, ...]
    edges: frozenset[tuple[str, str]]

    def render(self) -> str:
        declarations = tuple(node.declaration() for node in self.nodes)
        lines = ('```mermaid', 'flowchart LR', *declarations)
        edges = tuple(
            f'    {_identifier(source)} --> {_identifier(target)}'
            for source, target in sorted(self.edges)
        )
        return '\n'.join((*lines, *edges, '```'))


def _identifier(iri: str) -> str:
    digest = sha256(iri.encode()).hexdigest()
    return f'node_{digest}'


def _escape_label(text: str) -> str:
    return ''.join(
        character if character.isalnum() or character == ' ' else f'#{ord(character)};'
        for character in ' '.join(text.split())
    )
