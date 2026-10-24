"""Agent conversation admonitions."""

from pathlib import Path

from docs_macros.locations import CONVERSATIONS_DIR, github_url, indent_block

_MESSAGE_HEADING = """
**{speaker}**<span class="agent-message__source" markdown>[`{filename}`]({url})</span>
""".strip()


def conversation_paths(name: str) -> tuple[Path, Path]:
    directory = CONVERSATIONS_DIR / name
    prompt_path = directory / 'user.prompt'
    response_path = directory / 'codex.response'
    if not prompt_path.is_file() or not response_path.is_file():
        raise ValueError(f'Agent conversation does not exist: {name}')
    return prompt_path, response_path


def message_url(name: str, path: Path, repo_url: str) -> str:
    return github_url(Path('docs/conversations') / name / path.name, repo_url)


def message_heading(speaker: str, path: Path, url: str) -> str:
    return _MESSAGE_HEADING.format(
        speaker=speaker,
        filename=path.name,
        url=url,
    )


def agent_message(
    speaker: str,
    admonition: str,
    path: Path,
    name: str,
    repo_url: str,
) -> str:
    heading = message_heading(speaker, path, message_url(name, path, repo_url))
    body = indent_block(path.read_text().strip(), 4)
    return f'!!! {admonition} "{heading}"\n\n{body}\n'
