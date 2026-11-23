"""Rendering of saved GitHub metadata in Markdown table cells."""

from pathlib import Path
from xml.etree import ElementTree

import pytest
from markdown import markdown
from ruamel.yaml import YAML

from docs_macros import github


@pytest.fixture
def capture_path(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setattr(github, 'GITHUB_RESULTS_DIR', tmp_path)
    path = tmp_path / 'owner' / 'repo.yamlld'
    path.parent.mkdir()
    return path


@pytest.mark.parametrize(
    'repository',
    (
        'Owner/Repo',
        'https://github.com/Owner/Repo',
        'http://www.github.com/Owner/Repo.git',
        'https://github.com/Owner/Repo/blob/main/README.md?plain=1',
    ),
)
def test_github_stars_preserves_zero(capture_path: Path, repository: str) -> None:
    with capture_path.open('w', encoding='utf-8') as capture:
        YAML().dump({'stars': 0, 'observedAt': '2026-11-19T12:30:00Z'}, capture)

    assert github.github_stars(repository) == (
        '[0](https://github.com/owner/repo/stargazers "Observed 2026-11-19")'
    )
    assert github.github_latest_release('Owner/Repo') == ''


def test_github_macros_leave_missing_cells_blank(capture_path: Path) -> None:
    assert not capture_path.exists()
    assert github.github_stars('owner/repo') == ''
    assert github.github_latest_release('owner/repo') == ''


def test_github_release_escapes_table_cell(capture_path: Path) -> None:
    tag = 'v1.0_[RC]|<next>'
    release_url = (
        'https://github.com/owner/repo/releases/tag/v1.0_%5BRC%5D%7C%3Cnext%3E'
    )
    with capture_path.open('w', encoding='utf-8') as capture:
        YAML().dump(
            {
                'latestRelease': {
                    'tag': tag,
                    'url': release_url,
                    'publishedAt': '2026-11-18T18:00:00Z',
                },
            },
            capture,
        )

    table = ElementTree.fromstring(
        markdown(
            f'| Release |\n| --- |\n| {github.github_latest_release("owner/repo")} |',
            extensions=['tables'],
        ),
    )

    link = table.find('./tbody/tr/td/a')
    assert len(table.findall('./tbody/tr/td')) == 1
    assert link is not None
    assert link.text == tag
    assert link.attrib == {'href': release_url, 'title': 'Released 2026-11-18'}
    assert (link[0].tag, link[0].tail) == ('br', '2026-11-18')


@pytest.mark.parametrize(
    'repository', ('../repo', 'owner/..', 'https://example.com/owner/repo')
)
def test_github_stars_rejects_invalid_repository(repository: str) -> None:
    with pytest.raises(ValueError, match='Expected a GitHub repository'):
        github.github_stars(repository)
