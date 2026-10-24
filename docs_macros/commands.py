"""Shell and command admonitions for documentation macros."""

import json
import os
import subprocess
from pathlib import Path

from docs_macros.locations import ROOT_DIR, indent_block
from mkdocs_macros_sparqld import stop_server


def command_text(command: list[str]) -> str:
    return ' '.join(map(str, command))


def run_command(
    command: str | list[str],
    expected: str | None = None,
    environment: dict[str, str] | None = None,
    cwd: Path = ROOT_DIR,
    shell: bool = False,
) -> str:
    if isinstance(command, str):
        argv = ['/bin/sh', '-c', command]
    else:
        argv = command
    rendered_command = command_text(argv)
    try:
        completed_process = subprocess.run(
            argv,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd,
            env=environment,
        )
    except FileNotFoundError as error:
        stop_server()
        raise RuntimeError(
            f'Required documentation command `{argv[0]}` was not found.'
        ) from error
    except subprocess.CalledProcessError as error:
        stop_server()
        detail = (error.stderr or error.stdout).strip()
        raise RuntimeError(
            f'Documentation command failed: {rendered_command}\n{detail}'
        ) from error
    output = completed_process.stdout.strip()
    if expected and expected not in output:
        stop_server()
        raise RuntimeError(
            f'Documentation command did not return `{expected}`:\n{rendered_command}\n{output}',
        )
    return output


def shell(
    command: str,
    env: dict[str, str] | None = None,
    output_format: str | None = None,
    indent: int = 0,
) -> str:
    """Run a shell command and render its JSON output."""
    if output_format is None:
        output_format = json.__name__
    if output_format != json.__name__:
        raise ValueError(f'Unsupported shell output format: {output_format}')
    try:
        output = json.dumps(
            json.loads(
                run_command(
                    command,
                    environment={
                        **os.environ,
                        **{
                            name: str(env_value)
                            for name, env_value in (env or {}).items()
                        },
                    },
                    shell=True,
                )
            ),
            indent=2,
        )
    except json.JSONDecodeError as error:
        stop_server()
        raise RuntimeError(
            f'Documentation shell command did not return JSON: {command}'
        ) from error
    rendered = f'```console\n{command}\n```\n\n```{json.__name__} title="Response"\n{output}\n```'
    return indent_block(rendered, indent)


def command(command_text: str, indent: int = 0) -> str:
    body = f'```console\n{command_text}\n```'
    return f'!!! command "Command"\n\n{indent_block(body, indent + 4)}\n'
