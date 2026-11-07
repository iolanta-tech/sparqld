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

## :material-code-braces-box: Combined example

The combined project keeps these files in one directory. Paths are relative
to it:

=== "alice.yamlld"

{{ example_data('self-registering-context/combined/alice.yamlld', indent=4) }}

=== "contexts/person.yamlld"

{{ example_data('self-registering-context/combined/contexts/person.yamlld', indent=4) }}

=== "contexts/z-dollar.yamlld"

{{ example_data('self-registering-context/combined/contexts/z-dollar.yamlld', indent=4) }}

The dollar context here is a minimal fixture defining only `$id`, with an
example URI to keep it separate from the complete context.
For broader dollar-keyword authoring, use an explicitly acquired copy of the
[complete context](https://json-ld.org/contexts/dollar-convenience.jsonld).

The top-level `defines-context` member gives the context URI. It has no `@id`,
so `sparqld` takes the containing file as the subject. As source data, this
produces the registration triple. As a requested context, only its `@context`
member supplies definitions; the top-level registration metadata does not
become consumer data.
This follows the [JSON-LD context-processing algorithm](https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm).

This query reads Alice's name through the combined context:

{{ example_data('self-registering-context/queries/alice.rq') }}

{{ source('docs/examples/self-registering-context/results/alice.json', title='Expected result') }}

## :material-source-branch: Bootstrap and paths

- Request the [built-in context](built-in-context.md) for the registration
  terms. Do not define `defines-context` by hand. `sparqld` retries failed
  sources as registrations become available; filename order does not need to
  put declarations first.
- Keep registration dependencies acyclic. A file that needs its own canonical
  URI to interpret its registration cannot establish that registration.
- Include registration sources in the selected source patterns. Reading a body
  as a context does not load its registration metadata into the dataset.
- With the default document base, the `@id` of a registration in a separate
  file resolves from the source directory: `person.yamlld` in the separate
  layout becomes `sparqld:contexts/person.yamlld`. An explicit `@base` changes
  this resolution. Keep the resulting subject local and inside the served
  directory. A statement inside the body has no `@id` and ignores `@base`.
- Relative references **inside a requested context** resolve from its requested
  URI. For `https://example.org/contexts/combined/person`, `terms.yamlld` becomes
  `https://example.org/contexts/combined/terms.yamlld`, which needs its own registration.
  A sibling file alone does not establish that mapping. Absolute dependency
  URIs make this distinction explicit.

An array in `@context` composes contexts in order. It is distinct from
`@import`, whose referenced `@context` must be an object, not an array.

## :material-file-multiple-outline: Separate layout

The separate layout uses the same consumer and dollar definitions, with these
two files:

=== "contexts/person.yamlld"

{{ example_data('self-registering-context/separate/contexts/person.yamlld', indent=4) }}

=== "contexts/z-registration.yamlld"

{{ example_data('self-registering-context/separate/contexts/z-registration.yamlld', indent=4) }}

| Operation | Combined layout | Separate layout |
| --- | --- | --- |
| Edit context terms | Reloads consumers | Reloads consumers |
| Remove registration triple | Consumers fail | Consumers fail; body remains |
| Delete or corrupt person body | Registration disappears; consumers fail | Declaration remains; consumers fail |
| Preserve an upstream body exactly | Requires a separate project declaration | Keeps original bytes |

Both layouts passed startup, term edits, registration removal, person body deletion,
malformed YAML, and restoration checks, including restart comparisons.
The separate layout is useful for acquired contexts whose bytes must stay intact.

## :material-alert-outline: Current failure limits

Tests against `sparqld` 0.1.6 with `oxjsonld` 0.2.5 also reproduced a parser
assertion, `The remote context stack is invalid`:

- A recursive relative self-context panics initialization. The process remains
  alive, but its HTTP endpoint does not become available.
- Deleting the dollar fixture in the separate layout panics the watcher and
  leaves previously loaded consumer data visible. Starting that incomplete
  project also panics initialization.

The tested combined layout handles that dependency deletion and restoration.
This does not establish protection against every nested-context failure.
Resolve these parser failures before adopting either layout as an unrestricted
project standard. The regression tests retain the desired error behavior as
expected failures, restricted to the observed assertion.

## :material-swap-horizontal: Other processors

Other JSON-LD processors need an equivalent document-loader mapping or an
accessible canonical URI; the RDF registration is a `sparqld` convention.
YAML-LD-aware loading or YAML-to-JSON conversion is also required for `.yamlld`.
Context URLs remain offline in `sparqld`: registered URLs read local files;
unregistered URLs fail. Acquire or refresh remote bodies explicitly.
