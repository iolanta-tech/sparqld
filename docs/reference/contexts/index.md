---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/contexts/index.md
"@type": schema:TechArticle
hide: [toc]
name: JSON-LD contexts
title: Contexts
description: Local context files and remote context URIs registered in project RDF.
---

# :material-code-braces-box: Contexts

Every JSON-LD-derived source starts from the
[default context](default-context.md), which `sparqld` applies without a
request. A source declares any other term it uses in its own context, so the
same file keeps its meaning in another JSON-LD tool.

## :material-check-circle-outline: Supported contexts

| Context | Use |
| --- | --- |
| A relative `.jsonld` or `.yamlld` path | A context file under the served directory |
| An absolute context URI with a project registration | A local body named by [`sparqld:defines-context`](https://sparqld.iolanta.tech/defines-context), or a saved [`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) whose [`schema:contentUrl`](http://schema.org/contentUrl) is that URI |

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

`sparqld` does not download context URLs. The one exception is the
[default context](default-context.md), which `sparqld` embeds. A source that
names any other absolute context URI loads when project RDF links a local
`.json`, `.jsonld`, or `.yamlld` body to that URI through
[`sparqld:defines-context`](https://sparqld.iolanta.tech/defines-context).
A node of type [`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) with
[`schema:contentUrl`](http://schema.org/contentUrl) set to the remote URL
registers the file that contains it. The node can omit `@id`, or set `@id` to
the remote URL.
[:fontawesome-brands-github: `iolanta-tech/vitrify`](https://github.com/iolanta-tech/vitrify)
writes the named form. Multiple declarations may name the same body. Distinct
bodies for one context URI produce a load error.

The [context dependency example](context-dependencies.md) shows a declaration
that sorts after its consumer. A project that already requests the
dollar-convenience URL can register it the same way:

{{ example_data('dollar-convenience-context.jsonld') }}

See [Self-registering contexts](self-registering-contexts.md) for a saved
context in the same file as its term map, including bootstrap requirements and
current error-handling limits.

The [default context](default-context.md) `https://sparqld.iolanta.tech/contexts/v1.yamlld`
needs no registration and no request. It defines `defines-context` and the
dollar aliases.

## :material-shield-lock-outline: Context access

An unregistered absolute URL fails the source that requested it. A relative
path cannot escape the served directory. This keeps a dataset reproducible and
prevents a context change elsewhere from changing its meaning.
