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

The [built-in context](reference/contexts/built-in-context.md)
`https://sparqld.iolanta.tech/contexts/v1.jsonld`
defines the `sparqld:` prefix and the `defines-context` term. Request it in
every file that states `defines-context`. `sparqld` embeds this context and does
not download it.

Several declarations may name the same context URI and the same body.
Distinct bodies for one context URI produce a load error. Changing or
removing the registration reloads dependent sources.

See [Contexts](reference/contexts/index.md) for resolution rules and the
[context dependency example](reference/contexts/context-dependencies.md) for a
complete registration and removal workflow.
