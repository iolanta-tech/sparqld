"""Build-time sparqld process lifecycle for mkdocs-macros."""

from __future__ import annotations

import atexit
import shutil
import socket
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from urllib.request import Request, urlopen


@dataclass
class ServerState:
    """Configuration and process shared by the documentation macros."""

    project_directory: Path | None = None
    directory: Path | None = None
    binary: str | Path | None = None
    process: subprocess.Popen | None = None
    endpoint: str | None = None

    def wait_for_endpoint(self, process: subprocess.Popen, endpoint: str) -> str:
        """Wait for a ready endpoint or clean up a failed server startup."""
        deadline = time.monotonic() + _STARTUP_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            self.check_process(process)
            try:
                ready = self.endpoint_is_ready(endpoint)
            except OSError:
                time.sleep(0.1)
                continue
            if ready:
                return endpoint

        stop_server()
        raise RuntimeError('Timed out starting sparqld for documentation macros.')

    def check_process(self, process: subprocess.Popen) -> None:
        """Report a startup failure and clear the stopped process state."""
        if process.poll() is None:
            return
        stdout, stderr = process.communicate()
        self.process = None
        self.endpoint = None
        raise RuntimeError(
            'Could not start sparqld for documentation macros.\n'
            + (stderr.strip() or stdout.strip())
        )

    def endpoint_is_ready(self, endpoint: str) -> bool:
        """Check whether the landing page responds successfully."""
        with urlopen(endpoint, timeout=_PROBE_TIMEOUT_SECONDS) as response:
            return response.status == _HTTP_OK


server_state = ServerState()
_STARTUP_TIMEOUT_SECONDS = 30
_PROBE_TIMEOUT_SECONDS = 0.25
_HTTP_OK = 200


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
def configure(*, project_dir: Path, directory: Path, binary: str | Path) -> None:
    """Store paths used to launch sparqld for this MkDocs project."""
    server_state.project_directory = project_dir
    server_state.directory = directory
    server_state.binary = binary


def _resolve_binary() -> str:
    if server_state.binary is None or server_state.project_directory is None:
        raise RuntimeError(
            'mkdocs_macros_sparqld is not configured. Ensure the pluglet is listed under macros.modules.',
        )
    candidate = Path(server_state.binary)
    if candidate.is_absolute() and candidate.is_file():
        return str(candidate)
    relative = (server_state.project_directory / candidate).resolve()
    if relative.is_file():
        return str(relative)
    found = shutil.which(str(server_state.binary))
    if found:
        return found
    raise RuntimeError(
        f'sparqld binary `{server_state.binary}` was not found. Install sparqld on PATH or set extra.sparqld.binary to an executable path.',
    )


def _unused_port() -> int:
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def ensure_endpoint() -> str:
    """Start sparqld if needed and return its SPARQL endpoint URL."""
    if (
        server_state.process is not None
        and server_state.process.poll() is None
        and server_state.endpoint is not None
    ):
        return server_state.endpoint
    if server_state.directory is None or server_state.project_directory is None:
        raise RuntimeError(
            'mkdocs_macros_sparqld is not configured. Ensure the pluglet is listed under macros.modules.',
        )
    if not server_state.directory.is_dir():
        raise RuntimeError(
            f'sparqld directory does not exist: {server_state.directory}'
        )

    port = _unused_port()
    endpoint = f'http://127.0.0.1:{port}/'
    binary = _resolve_binary()
    server_state.process = subprocess.Popen(
        [
            binary,
            str(server_state.directory),
            '--host',
            '127.0.0.1',
            '--port',
            str(port),
            '--no-watch',
        ],
        cwd=server_state.project_directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    server_state.endpoint = endpoint

    return server_state.wait_for_endpoint(server_state.process, endpoint)


def stop_server() -> None:
    """Terminate the build-time sparqld process if it is running."""
    if server_state.process is None:
        return
    if server_state.process.poll() is None:
        server_state.process.terminate()
        try:
            server_state.process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            server_state.process.kill()
            server_state.process.communicate()
    server_state.process = None
    server_state.endpoint = None


def run_query(query: str) -> tuple[str, str]:
    """POST a SPARQL query and return `(content_type, body)`."""
    endpoint = ensure_endpoint()
    request = Request(
        endpoint,
        data=query.encode(),
        headers={'Content-Type': 'application/sparql-query'},
        method='POST',
    )
    try:
        with urlopen(request, timeout=10) as response:
            return response.headers.get_content_type(), response.read().decode()
    except OSError:
        stop_server()
        raise


atexit.register(stop_server)
