"""Typed roadmap nodes and deterministic Mermaid declarations."""

from dataclasses import dataclass
from hashlib import sha256
from typing import Literal


@dataclass(frozen=True)
class RoadmapNode:
    iri: str
    title: str

    def declaration(self) -> str:
        label = _escape_label(self.title)
        suffix = ':::bug' if isinstance(self, Bug) else ''
        return f'    {_identifier(self.iri)}("{label}"){suffix}'


@dataclass(frozen=True)
class Goal(RoadmapNode):
    kind: Literal['goal'] = 'goal'


@dataclass(frozen=True)
class Bug(RoadmapNode):
    kind: Literal['bug'] = 'bug'


@dataclass(frozen=True)
class Roadmap:
    nodes: tuple[RoadmapNode, ...]
    edges: frozenset[tuple[str, str]]

    def render(self) -> str:
        declarations = tuple(node.declaration() for node in self.nodes)
        lines = ('```mermaid', 'flowchart LR', *declarations)
        if any(isinstance(node, Bug) for node in self.nodes):
            lines += ('    classDef bug fill:#fee2e2,stroke:#dc2626,color:#7f1d1d',)
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
