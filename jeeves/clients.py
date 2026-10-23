"""SPARQL client checks."""

import typer


def verify_clients():
    """Run the SPARQL client integration tests and write their record."""
    import pytest

    code = pytest.main(
        [
            '-m',
            'integration',
            '-o',
            'addopts=',
            '--tb=short',
            'tests/integration',
        ],
    )
    if code:
        raise typer.Exit(code=int(code))
    typer.echo('Wrote docs/results/client-compatibility.yamlld')
