"""Quote checking from an independently installed sparqld skill."""

import shlex
import shutil
import subprocess
from pathlib import Path

import pytest


@pytest.fixture
def installed_command(tmp_path: Path) -> list[str]:
    """Resolve the documented command from a separately copied skill."""
    installed = tmp_path / 'installed-skills' / 'sparqld'
    shutil.copytree(
        Path(__file__).resolve().parents[1] / 'skills' / 'sparqld', installed
    )
    instructions = (installed / 'SKILL.md').read_text()
    after_command = instructions.split('```sh\nyamllint ', 1)[1]
    documented = after_command.split('\n```', 1)[0]
    configuration = installed / 'yamllint.yaml'
    return shlex.split(
        f'yamllint {documented}'.replace(
            '<absolute-skill-directory>/yamllint.yaml', str(configuration)
        ).replace(' <written-files>', '')
    )


@pytest.fixture
def knowledge_base(tmp_path: Path) -> Path:
    """Create the unrelated working directory and quote-style fixtures."""
    directory = tmp_path / 'knowledge-base'
    directory.mkdir()
    (directory / 'accepted.yamlld').write_text(
        '"@context": {}\n"@id": urn:example:item\n'
    )
    (directory / 'rejected.yamlld').write_text(
        "'@context': {}\n'@id': 'urn:example:item'\n"
    )
    return directory


def _run(command: list[str], directory: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=directory,
        capture_output=True,
        text=True,
        check=False,
    )


def test_installed_skill_discovers_yamlld(
    installed_command: list[str],
    knowledge_base: Path,
) -> None:
    """Find YAML-LD files through the packaged configuration."""
    discovery = _run(
        [*installed_command, '--list-files', str(knowledge_base)], knowledge_base
    )
    expected = {str(path) for path in knowledge_base.iterdir()}
    assert discovery.returncode == 0, discovery.stderr
    assert set(discovery.stdout.splitlines()) == expected


@pytest.mark.parametrize(
    ('filename', 'status'), [('accepted.yamlld', 0), ('rejected.yamlld', 1)]
)
def test_installed_skill_quote_check(
    installed_command: list[str],
    knowledge_base: Path,
    filename: str,
    status: int,
) -> None:
    """Check quote style using only separately installed skill resources."""
    checked = _run([*installed_command, str(knowledge_base / filename)], knowledge_base)
    assert checked.returncode == status, checked.stdout + checked.stderr
    if status:
        assert 'quoted-strings' in checked.stdout
