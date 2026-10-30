---
"@context":
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": https://sparqld.iolanta.tech/context-file
"@type": rdf:Property
title: "sparqld:context-file"
description: Link a context URI to its local body file.
---

# :material-file-link: `sparqld:context-file`

**IRI:** <https://sparqld.iolanta.tech/context-file>

Link an absolute context URI to the local file that supplies its JSON-LD
context. The file must remain inside the served directory and use a `.json`,
`.jsonld`, or `.yamlld` extension.

The `sparqld:` vocabulary prefix expands to `https://sparqld.iolanta.tech/`.
In this YAML-LD registration, the body path resolves relative to the
declaration file:

{{ example_data('registered-context/project/z-context.yamlld') }}

Sources that request `https://example.org/yaml-ctx` use `body.yamlld` after
this declaration loads. `sparqld` reads the local body without downloading
the context URI.

Several declarations may name the same body. Distinct body targets for one
context URI produce a load error. Changing or removing the registration
reloads dependent sources.

See [Contexts](reference/contexts.md) for resolution rules and the
[context dependency example](reference/context-dependencies.md) for a
complete registration and removal workflow.
