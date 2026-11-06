---
"@context":
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": https://sparqld.iolanta.tech/defines-context
"@type": rdf:Property
title: "sparqld:defines-context"
description: Link a local body file to the context URI it defines.
---

# :material-file-link: `sparqld:defines-context`

**IRI:** <https://sparqld.iolanta.tech/defines-context>

Link a local file to the absolute context URI that the file defines. The
subject is the body file. The object is the context URI. The file must remain
inside the served directory and use a `.json`, `.jsonld`, or `.yamlld`
extension.

The `sparqld:` vocabulary prefix expands to `https://sparqld.iolanta.tech/`.
In this YAML-LD registration, the subject path resolves relative to the
declaration file:

{{ example_data('registered-context/project/z-context.yamlld') }}

Sources that request `https://example.org/yaml-ctx` use `body.yamlld` after
this declaration loads. `sparqld` reads the local body without downloading
the context URI.

## :material-file-check-outline: Statement inside the body

A body can define its own context URI. Omit `@id` in that statement.
`sparqld` takes the file itself as the subject:

{{ example_data('self-registering-context/combined/contexts/person.yamlld') }}

The embedded context `urn:sparqld:context:v1` defines the `sparqld:` prefix
and the `defines-context` term. A file can request it instead of defining the
term inline. `sparqld` serves it from memory and needs no registration.

Several declarations may name the same context URI and the same body.
Distinct bodies for one context URI produce a load error. Changing or
removing the registration reloads dependent sources.

See [Contexts](reference/contexts.md) for resolution rules and the
[context dependency example](reference/context-dependencies.md) for a
complete registration and removal workflow.
