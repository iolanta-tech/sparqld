---
title: Vault-LD
---

<!-- vale sparqld.HeadingAcronyms = NO -->
# Vault-LD
<!-- vale sparqld.HeadingAcronyms = YES -->

[Vault-LD](https://vault-ld.org/) leverages [:simple-yaml: YAML-LD](https://www.w3.org/TR/yaml-ld-10/) notation in Markdown frontmatter blocks to implement [LLMWiki](TODO: ?) pattern. It was one of the inspirations for `sparqld`. Thanks The Knowledge Graph Guys for implementing it!

Let's compare these two tools.

```mermaid
{% include 'project/decisions/implement-live-file-backed-sparql-access-with/vault-ld.mmd' %}
```

<div class="grid" markdown>

!!! quote "Vault-LD"

!!! quote "sparqld"

</div>

## Workflow

### Installation

<div class="grid" markdown>

!!! warning "Prepare a repository script"

    The reference exporter is a repository script with
    [:simple-python: Python](https://www.python.org/) dependencies.
    The exporter has no package on [:simple-pypi: PyPI](https://pypi.org/).
    Clone its checkout & install those dependencies before running it.

!!! success "Install with Cargo"

    Install the published package with
    [:simple-rust: Cargo](https://doc.rust-lang.org/cargo/), as shown in
    [Starting in CLI](../../../quickstart/cli.md).

</div>

??? example "Reproducing examples on this page"

    Start at your `sparqld` checkout root with Python 3 &
    [:simple-git: Git](https://git-scm.com/) installed.
    The commands clone [:fontawesome-brands-github: `The-Knowledge-Graph-Guys/vault-ld`](https://github.com/The-Knowledge-Graph-Guys/vault-ld)
    into a sibling directory, select the tested revision & install its dependencies.
    If that checkout already exists, skip the clone command.

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/prepare-exporter.sh', indent=4, title='Prepare the exporter') }}

    Run the diagram's command in the same terminal, from the Vault-LD checkout root.
    It writes `data.ttl` & `schema.ttl` under `build/wiki-link/`.

    To reproduce the comparisons, first prepare the `sparqld` development environment
    in its checkout's `.venv`, with `j` & the project dependencies installed.
    Continue in the same terminal with these commands.
    They return to `sparqld` & activate its environment.
    Run the final command alone to check existing results without regenerating them.

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/compare-exporter.sh', indent=4, title='Generate & check comparisons') }}

### Querying

<div class="grid" markdown>

!!! warning "Batch export & load"

    The reference exporter needs a batch export, a separate triple store & a load step.
    Refreshing results requires repeating export & load after edits.

!!! success "Query after initial load"

    After the initial directory load, query through the
    [HTTP endpoint](../../../reference/api/index.md) or
    [:simple-modelcontextprotocol: Model Context Protocol (MCP)](../../../reference/agents/mcp.md).
    The [file watcher](../../../reference/index.md) loads subsequent edits.

</div>

### Agent skills

<div class="grid" markdown>

!!! warning "Repository guidance"

    The inspected repository revision contains contributor guidance in
    [:fontawesome-brands-github: `AGENTS.md`](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/AGENTS.md).
    No reusable agent skill was found in that revision.

!!! success "Query, write & review"

    The [:fontawesome-brands-github: `SKILL.md`](https://github.com/iolanta-tech/sparqld/blob/main/skills/sparqld/SKILL.md)
    guides agents through queries, YAML-LD writing & data reviews.
    Follow [Agents](../../../reference/agents/index.md) to install it.

</div>

## Data model

### Graph organization

<div class="grid" markdown>

!!! info "Triples"

    The reference exporter writes [Turtle](https://www.w3.org/TR/turtle/) triples
    without named graphs.

!!! info "Source graphs"

    Each source file has a [named graph](../../../reference/file-catalog.md).
    Embedded graphs retain source-scoped names. Default queries use the union of named graphs.

</div>

### Wiki links

<div class="grid" markdown>

!!! warning "Preprocessing required"

    [:fontawesome-brands-github: `context.jsonld`](https://github.com/iolanta-tech/sparqld/blob/main/docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/wiki-link/context.jsonld)
    defines `companion` as an IRI-valued property. The exporter resolves
    `[[proxima-centauri]]` to the identifier declared in the separate
    [:fontawesome-brands-github: `proxima-centauri.md`](https://github.com/iolanta-tech/sparqld/blob/main/docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/wiki-link/proxima-centauri.md).

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/wiki-link/alpha-centauri.md', indent=4) }}

!!! success "Reference by RDF IRIs"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/wiki-link-comparison/rdf-iri.yamlld', indent=4) }}

</div>

### Input formats

<div class="grid" markdown>

!!! info "Typed Markdown notes"

    The [exporter reads Markdown front matter](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/scripts/vault_to_rdf.py#L470)
    from notes that declare a type.

!!! info "Markdown & standalone documents"

    `sparqld` loads Markdown front matter, standalone YAML-LD & standalone
    [:simple-json: JSON-LD](https://www.w3.org/TR/json-ld11/).
    See [Supported formats](../../../reference/formats.md) for the full list.

</div>

## Conformance

### Context placement

<div class="grid" markdown>

!!! warning "External contexts only"

    [Vault-LD §4.2](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/SPEC.md#42-the-context-is-shared-and-external-by-design)
    requires contexts outside notes. The exporter ignores inline contexts,
    contrary to JSON-LD & YAML-LD.
    The exporter reads [local JSON-LD context files](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/scripts/vault_to_rdf.py#L150)
    & skips remote references.

!!! info "Inline, local or registered contexts"

    Declare an inline context or reference a [local file](../../../reference/contexts/project-contexts.md)
    or a [registered context IRI](../../../reference/contexts/remote-contexts.md).

</div>

### Vocabulary expansion

<div class="grid" markdown>

!!! failure "Exporter discrepancy"

    The tested exporter mishandles `@vocab`, contrary to its specification.

!!! success "Standard expansion"

    JSON-LD vocabulary expansion gives bare terms their context-defined identifiers.

</div>

This [Alpha Centauri](https://dbpedia.org/resource/Alpha_Centauri) example compares
Vault-LD's exporter with [:simple-python: RDFLib](https://rdflib.readthedocs.io/).

!!! info "Tested exporter revision"

    The saved results use
    [:fontawesome-brands-github: `vault_to_rdf.py`](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/scripts/vault_to_rdf.py).
    Both processors receive the same frontmatter & folder context.
    The JSON-LD comparison supplies the context explicitly.

JSON-LD uses `@vocab` for the bare type `Star` & property `name`, as the
specification's
[`@vocab` example](https://github.com/The-Knowledge-Graph-Guys/vault-ld/blob/025e71be8d810e387dc451c920e05472d1c47170/SPEC.md#appendix-b-lifting-an-okf-bundle-compatibility-profile)
shows.
The expected result identifies Alpha Centauri as
[`dbo:Star`](https://dbpedia.org/ontology/Star) & preserves its
[`dbo:name`](https://dbpedia.org/ontology/name).
The tested exporter instead uses `urn:centauri:Star` & drops the name.

=== "Note"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/vocabulary/alpha-centauri.md', indent=4) }}

=== "Context"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/vocabulary/context.jsonld', indent=4) }}

=== "Results"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/vocabulary/vault-ld.ttl', indent=4, title='Vault-LD output') }}

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/vocabulary/rdflib.ttl', indent=4, title='JSON-LD output (RDFLib)') }}

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/vocabulary/exporter-warnings.txt', indent=4, title='Exporter warnings') }}

The same semantic input, represented as JSON-LD, also produces both triples in
`sparqld`. This checks standard JSON-LD loading. Vault-LD note lookup needs a separate test.

=== "sparqld input"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/sparqld-vocabulary/alpha-centauri.jsonld', indent=4) }}

=== "Query"

{{ source('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/sparqld-vocabulary/query.rq', indent=4) }}

=== "sparqld result"

{{ stored_sparql('docs/project/decisions/implement-live-file-backed-sparql-access-with/examples/sparqld-vocabulary/query.rq') | sparql_table | indent(4, true) }}

