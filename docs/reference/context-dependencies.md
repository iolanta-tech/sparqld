---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/context-dependencies.md
"@type": schema:TechArticle
title: Context dependency example
description: Resolve a remote JSON-LD context from a local body declared in project RDF.
---

# :material-link-variant: Context dependency example

Run this example from a `sparqld` checkout with the `sparqld` executable on
`PATH`. The project contains four files:

=== "alice.jsonld"

{{ example_data('recursive-context/project/alice.jsonld', indent=4) }}

=== "context.jsonld"

{{ example_data('recursive-context/project/context.jsonld', indent=4) }}

=== "remote.jsonld"

{{ example_data('recursive-context/project/remote.jsonld', indent=4) }}

=== "example.jsonld"

{{ example_data('recursive-context/project/example.jsonld', indent=4) }}

Alice uses a local context that imports `https://example.org/ctx`.
`remote.jsonld` links that URI to `example.jsonld` through
[`sparqld:context-file`](https://sparqld.iolanta.tech/context-file).
`sparqld` loads Alice after it has that declaration, even though Alice sorts
before `remote.jsonld`.

A YAML-LD project with the same shape, using `https://example.org/yaml-ctx`,
lives in `docs/examples/registered-context/project`:

=== "alice.yamlld"

{{ example_data('registered-context/project/alice.yamlld', indent=4) }}

=== "z-context.yamlld"

{{ example_data('registered-context/project/z-context.yamlld', indent=4) }}

=== "body.yamlld"

{{ example_data('registered-context/project/body.yamlld', indent=4) }}

{{ example_data('registered-context/queries/alice.rq') }}

{{ source('docs/examples/registered-context/results/alice.json', title='Expected result') }}

## :material-play-outline: Inspect the initial state

Start the server. Keep this terminal running:

{{ example_data('recursive-context/serve.sh') }}

In another terminal, run the dependency query:

{{ example_data('recursive-context/query.sh') }}

{{ source('docs/queries/context-dependencies.rq') }}

{{ source('docs/examples/recursive-context/results/initial/dependencies.json', title='Expected result') }}

Additional queries check the declaration, failed sources, and Alice's data:

=== "Declaration"

{{ example_data('recursive-context/queries/declaration.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/declaration.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ example_data('recursive-context/queries/failures.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/failures.json', indent=4, title='Expected result') }}

=== "Alice"

{{ example_data('recursive-context/queries/alice.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/alice.json', indent=4, title='Expected result') }}

## :material-refresh: Remove the declaration

Delete the declaration while the server runs:

{{ example_data('recursive-context/recover.sh') }}

The watcher reloads Alice and the importing context. Repeat the same queries.
The declaration ASK fails, both sources record load errors, and Alice's name
is no longer queryable:

=== "Dependencies"

{{ source('docs/examples/recursive-context/results/unregistered/dependencies.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ source('docs/examples/recursive-context/results/unregistered/failures.json', indent=4, title='Expected result') }}

=== "Alice"

{{ source('docs/examples/recursive-context/results/unregistered/alice.json', indent=4, title='Expected result') }}

After the demonstration, restore the declaration:

{{ example_data('recursive-context/restore.sh') }}
