"""mkdocs-macros pluglet: build-time sparqld queries for documentation."""

from mkdocs_macros_sparqld.configuration import configure as configure
from mkdocs_macros_sparqld.macros import SparqldMacros as SparqldMacros
from mkdocs_macros_sparqld.macros import __all__ as __all__
from mkdocs_macros_sparqld.macros import define_env as define_env
from mkdocs_macros_sparqld.macros import on_post_build as on_post_build
from mkdocs_macros_sparqld.server import ensure_endpoint as ensure_endpoint
from mkdocs_macros_sparqld.server import run_query as run_query
from mkdocs_macros_sparqld.server import stop_server as stop_server
