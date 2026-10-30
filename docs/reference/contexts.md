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
description: Local context files and remote context URIs registered in project RDF.
---

# :material-code-braces-box: Contexts

Every JSON-LD-derived source declares the terms and aliases it uses in its own
context, so the same file keeps its meaning in another JSON-LD tool.

## :material-check-circle-outline: Supported contexts

| Context | Use |
| --- | --- |
| A relative `.jsonld` or `.yamlld` path | A context file under the served directory |
| An absolute context URI with a project registration | A local body linked through [`sparqld:context-file`](sparqld:context-file) |

Relative contexts resolve from the source's directory. They may also use
`@import` to reference another relative context. Every referenced file must
remain inside the directory served by `sparqld`.

When a local context file or a registered body changes, `sparqld` reloads every
source that requested that context.

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

## :material-link-variant: Remote context URIs

`sparqld` does not download context URLs. A source that names an absolute
context URI loads when project RDF links that URI through
[`sparqld:context-file`](sparqld:context-file) to a local `.json`, `.jsonld`,
or `.yamlld` body. Multiple declarations may name the same body. Distinct
body targets for one context URI produce a load error.

The [context dependency example](context-dependencies.md) shows a declaration
that sorts after its consumer. The served `docs/examples` tree registers the
dollar-keyword URL the same way:

{{ example_data('dollar-convenience-context.jsonld') }}

## :material-shield-lock-outline: Context access

An unregistered absolute URL fails the source that requested it. A relative
path cannot escape the served directory. This keeps a dataset reproducible and
prevents a context change elsewhere from changing its meaning.
