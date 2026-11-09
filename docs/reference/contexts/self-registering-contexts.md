---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/contexts/self-registering-contexts.md
"@type": schema:TechArticle
title: Self-registering contexts
description: Combine a local context body and its project registration, with explicit bootstrap and resolution constraints.
---

# :material-file-link-outline: Self-registering contexts

A `.yamlld` file can define a context and register its canonical URI against
itself. Use this optional layout for project-owned contexts whose registration
can load independently. It is not yet a general recommendation for every
context: the parser has error-handling limits described below.

## :material-code-braces-box: Example

The project keeps these files in one directory. Paths are relative to it:

=== "alice.yamlld"

{{ example_data('self-registering-context/combined/alice.yamlld', indent=4) }}

=== "contexts/person.yamlld"

{{ example_data('self-registering-context/combined/contexts/person.yamlld', indent=4) }}

=== "contexts/z-dollar.yamlld"

{{ example_data('self-registering-context/combined/contexts/z-dollar.yamlld', indent=4) }}

The dollar context here is a minimal fixture defining only `$id`. It shows a
context body that depends on another registered context. The
[default context](default-context.md) already defines `$id` and the other
dollar aliases for every source.

`contexts/person.yamlld` records a saved context. The node has type
[`jsonld:Context`](http://www.w3.org/ns/json-ld#Context), and
[`schema:contentUrl`](http://schema.org/contentUrl) is the remote URL.
`sparqld` registers the file that contains the node. The node is the saved
copy, and the file is the graph. `contexts/z-dollar.yamlld` uses the same
registration with `@id` set to the remote URL.
[:fontawesome-brands-github: `iolanta-tech/vitrify`](https://github.com/iolanta-tech/vitrify)
writes that named form. Both files register.

This query lists each saved context and the file that registers it:

{{ source('docs/queries/materialized-contexts.rq') }}

As a requested context, only the `@context` member supplies term definitions.
This follows the [JSON-LD context-processing algorithm](https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm).

This query reads Alice's name through the combined context:

{{ example_data('self-registering-context/queries/alice.rq') }}

{{ source('docs/examples/self-registering-context/results/alice.json', title='Expected result') }}

## :material-source-branch: Bootstrap and paths

- Write [`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) and
  [`schema:contentUrl`](http://schema.org/contentUrl) as full IRIs on a saved
  context. `sparqld` retries failed sources as registrations become available;
  filename order does not need to put registrations first.
- Keep registration dependencies acyclic. A file that needs its own canonical
  URI to interpret its registration cannot establish that registration.
- Include registration sources in the selected source patterns. Reading a body
  as a context does not load its registration metadata into the dataset.
- For a saved context, the registered file is the graph that contains the node,
  so `@id` and `@base` do not select a different file.
- Relative references **inside a requested context** resolve from its requested
  URI. For `https://example.org/contexts/combined/person`, `terms.yamlld` becomes
  `https://example.org/contexts/combined/terms.yamlld`, which needs its own registration.
  A sibling file alone does not establish that mapping. Absolute dependency
  URIs make this distinction explicit.

An array in `@context` composes contexts in order. It is distinct from
`@import`, whose referenced `@context` must be an object, not an array.

## :material-alert-outline: Current failure limits

Tests against `sparqld` 0.1.6 with `oxjsonld` 0.2.5 also reproduced a parser
assertion, `The remote context stack is invalid`:

- A recursive relative self-context panics initialization. The process remains
  alive, but its HTTP endpoint does not become available.

The tested layout handles nested-context deletion and restoration for its
fixtures. This does not establish protection against every nested-context
failure. Resolve these parser failures before adopting the layout as an
unrestricted project standard. The regression tests retain the desired error
behavior as expected failures, restricted to the observed assertion.

## :material-swap-horizontal: Other processors

Other JSON-LD processors need an equivalent document-loader mapping or an
accessible canonical URI; the RDF registration is a `sparqld` convention.
YAML-LD-aware loading or YAML-to-JSON conversion is also required for `.yamlld`.
Context URLs remain offline in `sparqld`: registered URLs read local files;
unregistered URLs fail. Acquire or refresh remote bodies explicitly.
