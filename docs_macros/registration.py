"""MkDocs macros for project documentation."""

from typing import Any

from docs_macros.adr import adr_metadata, example_code, universe_metadata
from docs_macros.commands import command, shell
from docs_macros.compatibility import client_compatibility
from docs_macros.decisions import Decision, decision_log
from docs_macros.environment import EnvironmentMacros
from docs_macros.locations import CLIENT_COMPATIBILITY_PATH

__all__ = ['CLIENT_COMPATIBILITY_PATH', 'Decision']


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
