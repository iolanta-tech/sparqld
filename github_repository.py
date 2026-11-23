"""Normalize GitHub repository URLs for shared capture paths."""

from re import IGNORECASE, fullmatch

_GITHUB_HOST = r'https?://(?:www\.)?github\.com'
_REPOSITORY_URL = (
    rf'{_GITHUB_HOST}/(?P<owner>[a-z0-9-]+)/(?P<name>[a-z0-9_.-]+)(?:[/?#].*)?'
)
_GITHUB_SITE_PATHS = frozenset(
    (
        'about',
        'account',
        'apps',
        'collections',
        'contact',
        'customer-stories',
        'enterprise',
        'events',
        'explore',
        'features',
        'issues',
        'login',
        'marketplace',
        'new',
        'notifications',
        'orgs',
        'organizations',
        'pricing',
        'pulls',
        'search',
        'security',
        'settings',
        'site',
        'sponsors',
        'stars',
        'topics',
        'trending',
        'users',
    ),
)


def repository_path(url: str) -> str | None:
    """Return the lowercase owner and repository of a GitHub URL."""
    match = fullmatch(_REPOSITORY_URL, url, IGNORECASE)
    if match is None or match['name'] in {'.', '..'}:
        return None
    owner = match['owner'].lower()
    if owner in _GITHUB_SITE_PATHS:
        return None
    name = match['name'].lower().removesuffix('.git')
    return f'{owner}/{name}' if name and name not in {'.', '..'} else None
