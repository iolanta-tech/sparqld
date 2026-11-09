---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/contexts/context-dependencies.md
"@type": schema:TechArticle
title: Context dependency example
description: Resolve a remote JSON-LD context from a local file that defines it.
---

# :material-link-variant: Context dependency example

An absolute context URI loads when a local file defines it. This project keeps
two files in one directory:

=== "alice.jsonld"

{{ example_data('recursive-context/project/alice.jsonld', indent=4) }}

=== "example.jsonld"

{{ example_data('recursive-context/project/example.jsonld', indent=4) }}

Alice requests the context `https://example.org/ctx`.
`example.jsonld` contains a
[`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) node whose
[`schema:contentUrl`](http://schema.org/contentUrl) is that URL, so `sparqld`
registers the file. `sparqld` loads Alice after it has that node, even though
Alice sorts before `example.jsonld`.

## :material-database-search-outline: Initial state

The file catalog records which contexts each source requested:

{{ source('docs/queries/context-dependencies.rq') }}

{{ source('docs/examples/recursive-context/results/initial/dependencies.json', title='Expected result') }}

Further queries check the declaration, failed sources, and Alice's data:

=== "Declaration"

{{ example_data('recursive-context/queries/declaration.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/declaration.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ example_data('recursive-context/queries/failures.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/failures.json', indent=4, title='Expected result') }}

=== "Alice"

{{ example_data('recursive-context/queries/alice.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/alice.json', indent=4, title='Expected result') }}

## :material-refresh: Without the registration

When `example.jsonld` loses its [`schema:contentUrl`](http://schema.org/contentUrl) value, the watcher reloads
Alice. The declaration ASK fails, Alice records a load error, and her name is
no longer queryable:

=== "Dependencies"

{{ source('docs/examples/recursive-context/results/unregistered/dependencies.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ source('docs/examples/recursive-context/results/unregistered/failures.json', indent=4, title='Expected result') }}

=== "Alice"

{{ source('docs/examples/recursive-context/results/unregistered/alice.json', indent=4, title='Expected result') }}

## :material-file-multiple-outline: Registration in a separate file

A YAML-LD project with the same shape, using `https://example.org/yaml-ctx`,
keeps the statement in its own file. There the statement names the body with
`@id`:

=== "alice.yamlld"

{{ example_data('registered-context/project/alice.yamlld', indent=4) }}

=== "z-context.yamlld"

{{ example_data('registered-context/project/z-context.yamlld', indent=4) }}

=== "body.yamlld"

{{ example_data('registered-context/project/body.yamlld', indent=4) }}

{{ example_data('registered-context/queries/alice.rq') }}

{{ source('docs/examples/registered-context/results/alice.json', title='Expected result') }}
