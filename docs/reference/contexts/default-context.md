---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/contexts/default-context.md
"@type": schema:TechArticle
hide: [toc]
name: Default context
title: Default context
description: The context that sparqld applies to every JSON-LD-derived source before the source's own context.
---

# :material-package-variant-closed: Default context

`sparqld` applies one default context to every JSON-LD, YAML-LD, and
Markdown-LD source. The context is the initial context of the expansion: a
source does not request it, and the source's own `@context` is processed after
it. A term or prefix that both define takes the source's definition. Turtle,
TriG, and other RDF formats have no `@context` and are unaffected.

`sparqld` embeds the context and does not download it. It answers from memory,
without a network connection, a project file, or a registration. The
documentation site publishes the same file at
`https://sparqld.iolanta.tech/contexts/v1.yamlld` for other processors.

Version 1 defines:

- the dollar aliases of the
  [dollar-convenience context](https://json-ld.org/contexts/dollar-convenience.jsonld),
  such as `$id` and `$type`;
- the `sparqld:` prefix, which expands to `https://sparqld.iolanta.tech/`;
- the [`defines-context`](https://sparqld.iolanta.tech/defines-context) term,
  which links a local file to the context URI that the file defines.

{{ source('docs/contexts/v1.yamlld', title='https://sparqld.iolanta.tech/contexts/v1.yamlld') }}

## :material-layers-outline: Overriding the default

`alice.yamlld` has no `@context`, yet `$id` and `$type` resolve.
`schema.yamlld` redefines `$id` as an ordinary property, as a JSON Schema
document would use it:

=== "alice.yamlld"

{{ example_data('default-context/project/alice.yamlld', indent=4) }}

=== "schema.yamlld"

{{ example_data('default-context/project/schema.yamlld', indent=4) }}

The same query lists the predicate of each statement. `urn:alice` uses the
default meaning of `$id` and `$type`; `urn:schema` uses its own `$id`:

{{ example_data('default-context/queries/triples.rq') }}

{{ source('docs/examples/default-context/results/triples.json', title='Expected result') }}

A `null` in the source's `@context` discards the default context together with
every definition before it.

## :material-alert-outline: Keywords and the `sparqld:` prefix

Use `$id` and `$type` in data. Keep `@context` and the raw `@id`, `@type`, and
`@container` keywords in context definitions.

Because the default context defines the `sparqld` prefix, a compact IRI such as
`sparqld:contexts/person.yamlld` expands to
`https://sparqld.iolanta.tech/contexts/person.yamlld`. A source that names a
local file by its `sparqld:` graph IRI sets `"sparqld": null` in its own
`@context` so that the IRI stays absolute.

## :material-link-variant: Explicit requests

A source may still list the context URL in its `@context`. Loading a source
that does so records a
[`jsonld:context`](https://www.w3.org/ns/json-ld#context) edge to
`https://sparqld.iolanta.tech/contexts/v1.yamlld` in the
[file catalog](../file-catalog.md). Applying the default records no edge.

An embedded context takes precedence over a project registration of the same
URL.
