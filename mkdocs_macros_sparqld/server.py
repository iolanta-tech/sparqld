"""Build-time sparqld process lifecycle for mkdocs-macros."""

from __future__ import annotations

import atexit
import shutil
import socket
import subprocess
import time
from pathlib import Path
from urllib.request import Request, urlopen

_project_dir: Path | None = None
_directory: Path | None = None
_binary: str | Path | None = None
_server: subprocess.Popen | None = None
_endpoint: str | None = None


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

# @todo(pdd-ld-integration): Serve PDD-LD output through sparqld.
# description: >
#   Add this repository's pdd-ld extractor declaration to sparqld.toml:
#
#   [[extractors]]
#   id = "pdd-ld"
#   command = "target/debug/pdd-ld"
#   patterns = ["**/*.py", "**/*.rs", "**/*.toml", "!target/**"]
#
#   Preserve pdd-ld's executable boundary: sparqld invokes it once per matching
#   relative path and only consumes its JSON-LD. Add source annotations that
#   exercise a cross-file `blocked-by` relationship. Query the live endpoint to
#   prove both resources, their source locations, and the RDF edge exist.
# tests:
#   - Add an end-to-end fixture with Python, Rust, and TOML puzzles.
#   - Build the real pdd-ld binary, configure sparqld with it, and query the
#     resulting dataset rather than mocking subprocess output.
#   - Cover pdd-ld's stderr JSON-LD/nonzero failure as a catalogued sparqld
#     extractor failure.
# blocked-by:
#   - extractor-contributions
#   - extractor-reload
#   - pdd-ld
def configure(*, project_dir: Path, directory: Path, binary: str | Path) -> None:
    """Store paths used to launch sparqld for this MkDocs project."""
    global _project_dir, _directory, _binary
    _project_dir = project_dir
    _directory = directory
    _binary = binary


def _resolve_binary() -> str:
    if _binary is None or _project_dir is None:
        raise RuntimeError(
            'mkdocs_macros_sparqld is not configured. '
            'Ensure the pluglet is listed under macros.modules.'
        )
    candidate = Path(_binary)
    if candidate.is_absolute() and candidate.is_file():
        return str(candidate)
    relative = (_project_dir / candidate).resolve()
    if relative.is_file():
        return str(relative)
    found = shutil.which(str(_binary))
    if found:
        return found
    raise RuntimeError(
        f'sparqld binary `{_binary}` was not found. '
        'Install sparqld on PATH or set extra.sparqld.binary to an executable path.'
    )


def _unused_port() -> int:
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def ensure_endpoint() -> str:
    """Start sparqld if needed and return its SPARQL endpoint URL."""
    global _server, _endpoint
    if _server is not None and _server.poll() is None and _endpoint is not None:
        return _endpoint
    if _directory is None or _project_dir is None:
        raise RuntimeError(
            'mkdocs_macros_sparqld is not configured. '
            'Ensure the pluglet is listed under macros.modules.'
        )
    if not _directory.is_dir():
        raise RuntimeError(f'sparqld directory does not exist: {_directory}')

    port = _unused_port()
    endpoint = f'http://127.0.0.1:{port}/'
    binary = _resolve_binary()
    _server = subprocess.Popen(
        [
            binary,
            str(_directory),
            '--host',
            '127.0.0.1',
            '--port',
            str(port),
            '--no-watch',
        ],
        cwd=_project_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    _endpoint = endpoint

    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if _server.poll() is not None:
            stdout, stderr = _server.communicate()
            _server = None
            _endpoint = None
            raise RuntimeError(
                'Could not start sparqld for documentation macros.\n'
                f'{stderr.strip() or stdout.strip()}'
            )
        try:
            with urlopen(endpoint, timeout=0.25) as response:
                if response.status == 200:
                    return endpoint
        except OSError:
            time.sleep(0.1)

    stop_server()
    raise RuntimeError('Timed out starting sparqld for documentation macros.')


def stop_server() -> None:
    """Terminate the build-time sparqld process if it is running."""
    global _server, _endpoint
    if _server is None:
        return
    if _server.poll() is None:
        _server.terminate()
        try:
            _server.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            _server.kill()
            _server.communicate()
    _server = None
    _endpoint = None


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
