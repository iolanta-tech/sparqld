"""Build-time sparqld process lifecycle for mkdocs-macros."""

from __future__ import annotations

import atexit
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from urllib.request import Request, urlopen

from mkdocs_macros_sparqld import process

# @todo(existing-sparqld-endpoint): Query an already running sparqld server.
# description: >
#   Support an optional `extra.sparqld.endpoint` MkDocs setting, for example
#   `http://127.0.0.1:7737/`. It selects an externally managed sparqld endpoint
#   instead of starting a documentation-only subprocess.
# configuration:
#   - endpoint must be an absolute http or https URL with an explicit port.
#   - Reject a configuration that supplies endpoint together with an explicit
#     directory or binary override; these settings apply only to managed mode.
# behavior:
#   - Extend configure() to retain the optional endpoint. In endpoint mode,
#     ensure_endpoint() verifies a 200 landing response before returning it.
#   - Never start, terminate, kill, or clear an externally managed endpoint.
#     Keep the current temporary localhost process, automatic port, and cleanup
#     behavior when endpoint is omitted.
#   - Continue exporting sparqld_port from the explicit endpoint port so the
#     existing curl and library documentation examples keep working.
# tests:
#   - Test endpoint validation and conflicts with explicit directory/binary.
#   - Test that endpoint mode performs its readiness request without spawning a
#     process and that stop_server() leaves the endpoint usable.
#   - Test the existing managed-server path unchanged.


# @todo(riddles-integration): Serve Riddles output through sparqld.
# description: >
#   Build or use the real `riddles` executable and start sparqld with
#   `--extractor riddles`. Use Python and Rust fixture files containing valid
#   riddle annotations and a cross-file blocked-by edge. Query the live endpoint
#   to prove every riddle's global IRI, type, headline, description, source URL,
#   line metadata, dependency edge, and host-owned contribution graph.
#
#   Prove that malformed Riddles input becomes a catalog failure without
#   blocking valid native or documentation sources. The fixture uses only
#   descriptor-driven registration and the real Riddles executable.
# tests:
#   - Query the real endpoint rather than mocking subprocess output.
#   - Cover malformed annotation and nonzero extraction failures as catalogued
#     extractor failures.
# blocked-by:
#   - extractor-contributions
#   - extractor-reload
#   - riddles-cli
#   - extractor-registration


@dataclass
class ServerState:
    """Configuration and processes shared by the documentation macros."""

    project_directory: Path | None = None
    directory: Path | None = None
    binary: str | Path | None = None
    servers: dict[Path, tuple[subprocess.Popen, str]] = field(default_factory=dict)

    def ensure_endpoint_for(self, directory: Path) -> str:
        """Start sparqld for `directory` if needed and return its endpoint URL."""
        resolved = directory.resolve()
        existing = self.servers.get(resolved)
        if existing is not None and existing[0].poll() is None:
            return existing[1]
        if self.project_directory is None or self.binary is None:
            raise RuntimeError(
                'mkdocs_macros_sparqld is not configured. Ensure the pluglet is listed under macros.modules.',
            )
        if not resolved.is_dir():
            raise RuntimeError(f'sparqld directory does not exist: {resolved}')
        binary = process.resolve_binary(self.project_directory, self.binary)
        started = process.start_sparqld(
            binary=binary,
            directory=resolved,
            project_directory=self.project_directory,
            port=process.unused_port(),
        )
        self.servers[resolved] = started
        return process.wait_until_ready(
            started[0],
            started[1],
            on_failure=lambda: self.stop_directory(resolved),
        )

    def ensure_endpoint(self) -> str:
        """Start sparqld for the configured docs directory if needed."""
        if self.directory is None:
            raise RuntimeError(
                'mkdocs_macros_sparqld is not configured. Ensure the pluglet is listed under macros.modules.',
            )
        return self.ensure_endpoint_for(self.directory)

    def stop_all(self) -> None:
        for running, _endpoint in list(self.servers.values()):
            process.stop_process(running)
        self.servers.clear()

    def post_query(self, endpoint: str, query: str) -> tuple[str, str]:
        request = Request(
            endpoint,
            data=query.encode(),
            headers={'Content-Type': 'application/sparql-query'},
            method='POST',
        )
        with urlopen(request, timeout=10) as response:
            return response.headers.get_content_type(), response.read().decode()

    def stop_directory(self, directory: Path) -> None:
        running = self.servers.pop(directory, None)
        if running is not None:
            process.stop_process(running[0])


server_state = ServerState()


def configure(*, project_dir: Path, directory: Path, binary: str | Path) -> None:
    """Store paths used to launch sparqld for this MkDocs project."""
    server_state.project_directory = project_dir
    server_state.directory = directory
    server_state.binary = binary


def ensure_endpoint_for(directory: Path) -> str:
    """Start sparqld for `directory` if needed and return its endpoint URL."""
    return server_state.ensure_endpoint_for(directory)


def ensure_endpoint() -> str:
    """Start sparqld for the configured docs directory if needed."""
    return server_state.ensure_endpoint()


def stop_server() -> None:
    """Terminate every build-time sparqld process."""
    server_state.stop_all()


def run_query(query: str) -> tuple[str, str]:
    """POST a SPARQL query to the configured docs endpoint."""
    endpoint = ensure_endpoint()
    try:
        return server_state.post_query(endpoint, query)
    except OSError:
        stop_server()
        raise


def run_query_in(directory: Path, query: str) -> tuple[str, str]:
    """POST a SPARQL query against a specific served directory."""
    resolved = directory.resolve()
    endpoint = ensure_endpoint_for(resolved)
    try:
        return server_state.post_query(endpoint, query)
    except OSError:
        server_state.stop_directory(resolved)
        raise


atexit.register(stop_server)
