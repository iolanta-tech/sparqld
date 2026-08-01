---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/context-files.md
"@type": schema:TechArticle
hide: [toc]
name: Dedicated context files
title: Context files
description: How sparqld applies shared JSON-LD contexts across a directory tree.
---

# :material-code-braces-box: Dedicated context files

Place `context.jsonld` or `context.yamlld` in a directory to share a JSON-LD
context among the linked-data documents below it.

## :material-folder-multiple: Scope and inheritance

{{ directory_tree('docs/examples') }}

- `alpha-centauri.yamlld`, `centaurus.md`, and `proxima-centauri-b.jsonld` all
  use `examples/context.yamlld`.
- Subdirectories inherit this context unless they provide their own dedicated
  context file.
- A nearer dedicated context replaces the inherited dedicated context for its
  entire subtree.
- A directory may use `context.jsonld` instead. If both variants exist,
  `context.jsonld` takes precedence.

Dedicated context files configure parsing and are not loaded as source graphs.
Changing one reloads the source files below its directory.

## :simple-yaml: Context syntax

Dedicated context files contain a JSON-LD document with an `@context` property.
YAML-LD is parsed as YAML 1.2.

{{ example_data('context.yamlld') }}

The context applies to JSON-LD, YAML-LD, and Markdown-LD sources. An `@context`
inside an individual source is applied afterward, so its term definitions take
precedence.

All JSON-LD-derived sources also receive the built-in
[dollar keyword aliases](formats.md#dollar-keyword-aliases) before the directory
and inline contexts are applied.

## :material-check-circle-outline: Supported context forms

- Inline contexts in JSON-LD, YAML-LD, and Markdown-LD sources.
- A dedicated `context.jsonld` or `context.yamlld` file containing an inline
  `@context`. `sparqld` injects it automatically according to the inheritance
  rules above.
- The built-in [dollar keyword aliases](formats.md#dollar-keyword-aliases).

## :material-close-circle-outline: Referenced contexts

An `@context` value that points to another document is not currently supported.
This includes both remote URLs and relative local file references:

```json
{"@context": "https://example.org/context.jsonld"}
```

```json
{"@context": "../context.jsonld"}
```

Use an inline context or a dedicated context file named exactly
`context.jsonld` or `context.yamlld` instead. This restriction also applies to
the `@context` inside a dedicated context file.

## :material-link-variant: Relative IRIs

Each source uses the internal IRI of its containing directory as the JSON-LD
base. The Markdown-LD example links to another file with a relative IRI:

{{ example_data('centaurus.md') }}

When `examples/` is served, its base is `sparqld:`, so the relative identifier
resolves within the served directory.

| Value | Expanded IRI |
| --- | --- |
| `alpha-centauri.yamlld` | `sparqld:alpha-centauri.yamlld` |
