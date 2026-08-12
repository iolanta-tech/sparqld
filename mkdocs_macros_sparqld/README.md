# mkdocs-macros-sparqld

[mkdocs-macros](https://mkdocs-macros-plugin.readthedocs.io/) pluglet that runs
[sparqld](https://github.com/iolanta-tech/sparqld) during `mkdocs build` /
`mkdocs serve` and expands SPARQL macros into rendered pages.

Full documentation:
[MkDocs integration](https://sparqld.iolanta.tech/reference/integrations/mkdocs/)

## Install

```console
pip install mkdocs-macros-sparqld
```

Also install the [`sparqld`](https://crates.io/crates/sparqld) binary and keep it
on `PATH`, unless you override `extra.sparqld.binary`.

## Configuration

Register the pluglet with mkdocs-macros:

```yaml
plugins:
  - macros:
      modules: [mkdocs_macros_sparqld]
```

Nothing else is required when `sparqld` is on `PATH` and the served tree is the
project's `docs/` directory.

Optional overrides under `extra.sparqld`:

```yaml
extra:
  sparqld:
    directory: data                    # default: docs
    binary: target/debug/sparqld       # default: sparqld on PATH
```

## Macros and filters

- `sparql('SELECT …')` — run a verbatim SPARQL query; `SELECT` returns a list of
  binding dicts
- `stored_sparql('path/from/mkdocs/root.rq')` — run a query file relative to the
  MkDocs project root
- `sparql_table` — Jinja filter that renders `SELECT` bindings as a Markdown
  table
- `sparqld_port` — Jinja variable containing the local endpoint port assigned
  during MkDocs setup

```markdown
{{ stored_sparql('docs/queries/names.rq') | sparql_table }}
```

## License

Apache-2.0 OR MIT
