---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/contexts.md
"@type": schema:TechArticle
hide: [toc]
name: JSON-LD contexts
title: Contexts
description: Context URLs and local context files supported by sparqld.
---

# :material-code-braces-box: Contexts

Every JSON-LD-derived source declares the terms and aliases it uses in its own
context, so the same file keeps its meaning in another JSON-LD tool.

## :material-check-circle-outline: Supported contexts

| Context | Use |
| --- | --- |
| [Dollar keyword aliases](https://json-ld.org/contexts/dollar-convenience.jsonld) | `$id`, `$type`, `$reverse`, and the other JSON-LD keyword aliases |
| A relative `.jsonld` or `.yamlld` path | Project-specific terms kept under the served directory |

The dollar-keyword URL identifies a bundled context. `sparqld` uses its local
copy and never requests it over the network. Put that URL in a document's
`@context` when using dollar aliases.

Relative contexts resolve from the source's directory. They may also use
`@import` to reference another relative context. Every referenced file must
remain inside the directory served by `sparqld`.

When a local context changes, `sparqld` reloads every source that reaches it
through `@context` or `@import`.

## :material-folder-outline: Local context files

Use a relative path in `@context` for a context stored beside a source or in a
subdirectory. A file named `context.jsonld` or `context.yamlld` is ordinary:
it is used only when a document explicitly names it. JSON-LD context files use
an `@context` member; YAML-LD context files express the same structure in YAML
1.2. Like other recognized files, context files are also loaded as source
graphs.

The Alpha Centauri source and its `context.yamlld` file live in the same
directory. The context defines the terms used by the source; the source names
that context explicitly:

=== "context.yamlld"

{{ example_code('context.yamlld', indent=4) }}

=== "alpha-centauri.yamlld"

{{ example_code('alpha-centauri.yamlld', indent=4) }}

## :material-shield-lock-outline: Context access

Other absolute URLs are rejected. `sparqld` does not fetch contexts from the
web, and a relative path cannot escape the served directory. This keeps a
dataset reproducible and prevents a context change elsewhere from changing its
meaning.
