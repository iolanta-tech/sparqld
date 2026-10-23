"""Clippy diagnostics limited to lines changed relative to HEAD."""

import json
from typing import Literal

import sh
from pydantic import BaseModel, ConfigDict, Field, ValidationError

from jeeves.diff import line_is_modified, relative_path
from jeeves.shell import command_output

_JSON_CONFIG = ConfigDict(extra='ignore')


class ClippySpan(BaseModel):
    """One source span in a rustc diagnostic."""

    model_config = _JSON_CONFIG

    file_name: str
    line_start: int
    column_start: int
    is_primary: bool


class ClippyDiagnostic(BaseModel):
    """A rustc warning or error."""

    model_config = _JSON_CONFIG

    level: Literal['warning', 'error']
    text: str = Field(alias='message')
    spans: list[ClippySpan] = Field(default_factory=list)


class CompilerMessage(BaseModel):
    """A cargo JSON event that carries a rustc diagnostic."""

    model_config = _JSON_CONFIG

    reason: Literal['compiler-message']
    diagnostic: ClippyDiagnostic = Field(alias='message')


def clippy_findings(modified: dict[str, set[int]]) -> list[str]:
    """Return Clippy diagnostics whose primary span is on a changed line."""
    output = command_output(
        sh.cargo,
        'clippy',
        '--quiet',
        '--message-format=json',
    )
    selected = []
    for row in str(output).splitlines():
        if not row.startswith('{'):
            continue
        rendered = clippy_row(json.loads(row), modified)
        if rendered:
            selected.append(rendered)
    return selected


def clippy_text(diagnostic: ClippyDiagnostic, primary: ClippySpan) -> str:
    location = relative_path(primary.file_name)
    line = primary.line_start
    column = primary.column_start
    text = diagnostic.text
    return f'{location}:{line}:{column}: {text}'


def clippy_row(payload: object, modified: dict[str, set[int]]) -> str | None:
    """Format one Clippy JSON message when it touches the diff."""
    try:
        parsed = CompilerMessage.model_validate(payload)
    except ValidationError:
        return None
    primary = next(
        (span for span in parsed.diagnostic.spans if span.is_primary),
        None,
    )
    if primary is None:
        return None
    if not line_is_modified(primary.file_name, primary.line_start, modified):
        return None
    return clippy_text(parsed.diagnostic, primary)
