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

`sparqld` embeds this context and applies it to every JSON-LD, YAML-LD, and
Markdown-LD source before the source's own `@context`. Sources do not fetch it.
Turtle and TriG have no `@context`. Published for other processors at
`https://sparqld.iolanta.tech/contexts/v1.yamlld`:

{{ source('docs/contexts/v1.yamlld', title='https://sparqld.iolanta.tech/contexts/v1.yamlld') }}

A `null` in `@context` discards the default and every definition before it.

Listing `https://sparqld.iolanta.tech/contexts/v1.yamlld` in `@context` records
a [`jsonld:context`](https://www.w3.org/ns/json-ld#context) edge in the
[file catalog](../file-catalog.md); applying the default does not. The embedded
context wins over a registration of the same URL.

[Contexts](index.md) · [Remote contexts](remote-contexts.md)
