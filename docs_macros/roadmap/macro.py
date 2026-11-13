"""MkDocs macro that queries and renders the roadmap."""

from collections.abc import Callable
from typing import Any

from docs_macros.roadmap.render import render_roadmap
from mkdocs_macros_sparqld.table import SelectBindings

ROADMAP_QUERY = 'docs/queries/roadmap.rq'


def roadmap(query: Callable[[str], SelectBindings]) -> str:
    """Query the stored roadmap and render its Mermaid graph."""
    bindings = query(ROADMAP_QUERY)
    if not isinstance(bindings, SelectBindings):
        raise TypeError('The roadmap query must return SELECT bindings')
    return render_roadmap(bindings)


class RoadmapMacro:
    """Bind roadmap rendering to one MkDocs query environment."""

    def __init__(self, env: Any) -> None:
        self._env = env

    def render(self) -> str:
        """Query the roadmap and render its Mermaid graph."""
        return roadmap(self._env.macros['stored_sparql'])
