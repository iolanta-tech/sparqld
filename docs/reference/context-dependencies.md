---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/context-dependencies.md
"@type": schema:TechArticle
title: Context dependency example
description: Inspect recursive context requests and recover a source through a local import.
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
The declaration maps that URI to `example.jsonld`, but graph-based context
resolution is not implemented. Both Alice and the importing context fail to
load. Their observed context requests remain in the [file catalog](file-catalog.md).

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

## :material-refresh: Recover through a local import

Replace the remote import with a direct local import:

{{ example_data('recursive-context/recovered-context.jsonld') }}

Apply this replacement while the server runs:

{{ example_data('recursive-context/recover.sh') }}

The watcher reloads the importing context and Alice. Repeat the same queries.
The declaration remains present, both error entries disappear, and Alice's
name becomes queryable:

=== "Dependencies"

{{ source('docs/examples/recursive-context/results/recovered/dependencies.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ source('docs/examples/recursive-context/results/recovered/failures.json', indent=4, title='Expected result') }}

=== "Alice"

{{ source('docs/examples/recursive-context/results/recovered/alice.json', indent=4, title='Expected result') }}

After the demonstration, restore the original example file:

{{ example_data('recursive-context/restore.sh') }}
