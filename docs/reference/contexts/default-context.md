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
description: The embedded JSON-LD context sparqld applies before each source's own @context.
---

# :material-package-variant-closed: Default context

`$type` worked on the [Contexts](index.md) page because `sparqld` embeds this
context and applies it to every JSON-LD, YAML-LD, and Markdown-LD source before
the source's own `@context`. Sources do not fetch it. Turtle and TriG have no
`@context`. Other processors can load the context file from
`https://sparqld.iolanta.tech/contexts/v1.yamlld`. See
[SPARQL prefixes](../sparql/prefixes.md) for the prefixes available to queries.

{{ source('docs/contexts/v1.yamlld', title='https://sparqld.iolanta.tech/contexts/v1.yamlld') }}

A `null` in `@context` discards the default and every definition before it.

Listing the address `https://sparqld.iolanta.tech/contexts/v1.yamlld` in
`@context` records a [`jsonld:context`](https://www.w3.org/ns/json-ld#context)
edge in the [file catalog](../file-catalog.md). Applying the embedded default
does not add this edge. The embedded context takes precedence over a
registration at the same address.

[Contexts](index.md) · [Remote contexts](remote-contexts.md)
