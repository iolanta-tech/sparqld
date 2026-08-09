---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/integrations/mkdocs.md
"@type": schema:TechArticle
hide: [toc]
name: MkDocs integration
title: MkDocs
description: Build-time sparqld queries for MkDocs through the mkdocs-macros-sparqld pluglet.
---

# :simple-materialformkdocs: MkDocs

[`mkdocs-macros-sparqld`](https://pypi.org/project/mkdocs-macros-sparqld/) is an
[mkdocs-macros](https://mkdocs-macros-plugin.readthedocs.io/) pluglet. During
`mkdocs build` or `mkdocs serve` it starts a read-only `sparqld` endpoint over a
configured directory and expands SPARQL macros into the rendered pages.

This documentation site uses the pluglet for its live SPARQL examples.

## :material-download-outline: Install

```console
pip install mkdocs-macros-sparqld
```

Also install the [`sparqld`](https://crates.io/crates/sparqld) binary and keep it
on `PATH` unless you override `extra.sparqld.binary` below.

## :material-file-cog-outline: Required configuration

Register the pluglet with mkdocs-macros:

```yaml
{% include 'reference/integrations/mkdocs-macros.yml' %}
```

Nothing else is required when `sparqld` is on `PATH` and the served tree is the
project's `docs/` directory.

## :material-tune-variant: Optional customization

Override defaults under `extra.sparqld` only when you need them:

```yaml
{% include 'reference/integrations/mkdocs-macros-extra.yml' %}
```

| Key | Default | When to set it |
| --- | --- | --- |
| `directory` | `docs` | The RDF tree is not `docs/` (for example `data/`) |
| `binary` | `sparqld` | The executable is not on `PATH`, or you want a project-local build such as `target/debug/sparqld` |
## :material-function-variant: Macros and filters

### `sparql`

Run a verbatim SPARQL query. A `SELECT` returns a list of binding dicts (one
dict per row, keys are variable names).

{% raw %}
```markdown
{{ sparql('SELECT ?name WHERE { ?s <https://schema.org/name> ?name } LIMIT 3') | sparql_table }}
```
{% endraw %}

{{ sparql('SELECT ?name WHERE { ?s <https://schema.org/name> ?name } LIMIT 3') | sparql_table }}

### `stored_sparql`

Run a `.rq` file whose path is relative to the MkDocs project root (the
directory that contains `mkdocs.yml`).

{% raw %}
```markdown
{{ stored_sparql('docs/queries/names.rq') | sparql_table }}
```
{% endraw %}

{{ stored_sparql('docs/queries/names.rq') | sparql_table }}

### `sparql_table` filter

Render `SELECT` bindings as a Markdown table. Pipe the result of `sparql` or
`stored_sparql`:

{% raw %}
```markdown
{{ stored_sparql('docs/queries/names.rq') | sparql_table }}
```
{% endraw %}

## :material-code-braces: Programmatic helpers

A local MkDocs macros module can reuse the same endpoint:

```python
{% include 'reference/integrations/helpers.py' %}
```
