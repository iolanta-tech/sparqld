---
"@context":
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": https://sparqld.iolanta.tech/defines-context
"@type": rdf:Property
title: "sparqld:defines-context"
description: Link a named local body file to the context URI it defines.
---

# :material-file-link: `sparqld:defines-context`

**IRI:** <https://sparqld.iolanta.tech/defines-context>

Link a named local file to the absolute context URI that the file defines. The
subject is the IRI of that body file. The object is the context URI. The file
must remain inside the served directory and use a `.json`, `.jsonld`, or
`.yamlld` extension.

In this YAML-LD registration, the subject path resolves relative to the
declaration file:

{{ example_data('registered-context/project/z-context.yamlld') }}

The [default context](reference/contexts/default-context.md) defines the
`defines-context` term, so this file does not define the term itself. Sources
that request `https://example.org/yaml-ctx` use `body.yamlld` after this
declaration loads. `sparqld` reads the local body without downloading the
context URI.

## :material-file-check-outline: Saved context file

A fetched context can register the file that saves it. That registration uses
[`jsonld:Context`](http://www.w3.org/ns/json-ld#Context) and
[`schema:contentUrl`](http://schema.org/contentUrl), described in
[Self-registering contexts](reference/contexts/self-registering-contexts.md).

Several declarations may name the same context URI and the same body.
Distinct bodies for one context URI produce a load error. Changing or
removing the registration reloads dependent sources.

See [Contexts](reference/contexts/index.md) for resolution rules and the
[context dependency example](reference/contexts/context-dependencies.md) for a
complete registration and removal workflow.
