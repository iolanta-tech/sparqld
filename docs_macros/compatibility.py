"""Recorded SPARQL client compatibility."""

from docs_macros.locations import CLIENT_COMPATIBILITY_PATH


def client_compatibility() -> str:
    """Render the recorded SPARQL client checks."""
    if not CLIENT_COMPATIBILITY_PATH.is_file():
        raise RuntimeError(
            'Client compatibility record is missing. Run `j verify-clients` before building the documentation.',
        )
    for line in CLIENT_COMPATIBILITY_PATH.read_text().splitlines():
        if line.startswith('summary:'):
            summary = line.split(':', 1)[1].strip().strip('"')
            return f'<!-- {summary} -->\n'
    raise RuntimeError(f'{CLIENT_COMPATIBILITY_PATH} does not include a summary.')
