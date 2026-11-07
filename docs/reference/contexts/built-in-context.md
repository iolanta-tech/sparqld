---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/contexts/built-in-context.md
"@type": schema:TechArticle
hide: [toc]
name: Built-in context
title: Built-in context
description: The context that sparqld embeds and serves for its own URL without downloading it.
---

# :material-package-variant-closed: Built-in context

`sparqld` embeds one context:
`https://sparqld.iolanta.tech/contexts/v1.jsonld`.
A source requests it like any other context URL.

`sparqld` does not download this URL. It has the context embedded and answers
the request from memory, without a network connection, a project file, or a
registration. The documentation site publishes the same file at that URL for
other JSON-LD processors, which fetch it from there.

An embedded context takes precedence over a project registration of the same
URL.

Version 1 defines:

- the `sparqld:` prefix, which expands to `https://sparqld.iolanta.tech/`;
- the [`defines-context`](https://sparqld.iolanta.tech/defines-context) term,
  which links a local file to the context URI that the file defines;
- the dollar aliases of the
  [dollar-convenience context](https://json-ld.org/contexts/dollar-convenience.jsonld),
  such as `$id` and `$type`.

{{ source('docs/contexts/v1.jsonld', title='https://sparqld.iolanta.tech/contexts/v1.jsonld') }}

## :material-link-variant: Registration terms

Request the context in every file that states `defines-context`. Do not define
the term by hand.

{{ example_data('registered-context/project/z-context.yamlld') }}

## :material-currency-usd: Dollar aliases

Use `$id` and `$type` in data. Keep `@context` and the raw `@id`, `@type`, and
`@container` keywords in context definitions. Each source that uses dollar
aliases must request this context, directly or through another context.

Loading a source that requests the context records a
[`jsonld:context`](https://www.w3.org/ns/json-ld#context) edge to
`https://sparqld.iolanta.tech/contexts/v1.jsonld` in the
[file catalog](../file-catalog.md).
