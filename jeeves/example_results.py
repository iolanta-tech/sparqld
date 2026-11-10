"""Refresh documented example SPARQL results as vitrify crates."""

from __future__ import annotations

import tempfile
from pathlib import Path

import sh
import typer

from jeeves.example_captures import Capture, captures
from jeeves.repository import PROJECT_ROOT
from jeeves.sparqld_serve import serve, wait_until_ready

VITRIFY_HINT = 'Install it with `cargo install vitrify`.'


def refresh_example_results() -> None:
    """Capture example SPARQL responses with vitrify into RO-Crate directories."""
    try:
        sh.vitrify('--version')
    except sh.CommandNotFound as error:
        message = f'`vitrify` was not found on PATH. {VITRIFY_HINT}'
        raise typer.BadParameter(message) from error
    binary = _build_sparqld()
    for capture in captures():
        _capture_state(capture, binary)


def _capture_state(capture: Capture, binary: Path) -> None:
    typer.echo(capture.label)
    with tempfile.TemporaryDirectory(prefix='sparqld-example-') as temporary:
        _write_crates(capture, binary, Path(temporary))
    relative = capture.crate_parent.relative_to(PROJECT_ROOT)
    typer.echo(f'  wrote {relative}')


def _write_crates(capture: Capture, binary: Path, temporary: Path) -> None:
    log = temporary / 'server.log'
    with serve(capture.prepare(temporary), binary, log) as server:
        wait_until_ready(server, capture.queries, log)
        _vitrify_queries(server.endpoint, capture)


def _build_sparqld() -> Path:
    target_directory = PROJECT_ROOT / 'target'
    sh.cargo.build(
        '--locked',
        '--target-dir',
        str(target_directory),
        _cwd=str(PROJECT_ROOT),
    )
    return target_directory / 'debug/sparqld'


def _vitrify_queries(endpoint: str, capture: Capture) -> None:
    for name, query_path in capture.queries:
        crate = capture.crate_parent / name
        crate.mkdir(parents=True, exist_ok=True)
        sh.vitrify(
            'sparql',
            '--endpoint',
            endpoint,
            '--query',
            str(query_path),
            '--format',
            'json',
            '--to',
            str(crate),
        )
