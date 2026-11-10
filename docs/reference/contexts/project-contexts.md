---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/contexts/project-contexts.md
"@type": schema:TechArticle
title: Project contexts
description: Define relative context files and optional self-registering project-owned context URIs.
---

# :material-folder-outline: Project contexts

=== "context.yamlld"

{{ example_code('context.yamlld', indent=4) }}

=== "alpha-centauri.yamlld"

{{ example_code('alpha-centauri.yamlld', indent=4) }}

Relative `@context` paths resolve from the source's directory. `@import` may
name another relative context. Every path must stay under the served directory.
Context files load as source graphs. Changing one reloads its dependents.

Remote URLs: [vitrify](remote-contexts.md).

## :material-file-link-outline: Self-registering project URIs

Optional: one file holds the term map and registers a project-owned URI.
Parser limits below apply.

=== "alpha-centauri.yamlld"

{{ example_data('self-registering-context/combined/alpha-centauri.yamlld', indent=4) }}

=== "contexts/person.yamlld"

{{ example_data('self-registering-context/combined/contexts/person.yamlld', indent=4) }}

=== "contexts/z-dollar.yamlld"

{{ example_data('self-registering-context/combined/contexts/z-dollar.yamlld', indent=4) }}

`z-dollar.yamlld` is only a nested-registration fixture. Ordinary sources get
`$id` from the [default context](default-context.md).

{{ source('docs/queries/materialized-contexts.rq') }}

Term definitions come only from the `@context` member
([JSON-LD context processing](https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm)).

{{ source('docs/examples/self-registering-context/queries/alpha-centauri.rq') }}

{{ stored_sparql('docs/examples/self-registering-context/queries/alpha-centauri.rq') | sparql_table }}

## :material-source-branch: Bootstrap and paths

- Write [`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) and
  [`schema:contentUrl`](http://schema.org/contentUrl) as full IRIs.
  `sparqld` retries failed sources when registrations appear. Filename order
  does not matter.
- Registration dependencies must be acyclic.
- Include registration sources in `--pattern` selection. Reading a body as a
  context does not load its registration triples.
- The registered graph is the file that contains the node. Neither `@id` nor
  `@base` selects another file.
- Relative references inside a requested context resolve from that context's
  requested URI, so they need their own registrations.

An `@context` array composes in order. `@import` requires an object, not an
array.

## :material-pause-circle-outline: Current failure limits

With `sparqld` 0.1.6 and `oxjsonld` 0.2.5, a recursive relative self-context
triggers `The remote context stack is invalid`. The process stays up, but the
HTTP endpoint never becomes ready. Keep this layout restricted until an update
resolves the error. Regression tests preserve the expected failure.
