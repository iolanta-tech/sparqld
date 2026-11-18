# YAML-LD rules

**LD01**: Express the intended facts with correct types, predicates, relationship
  directions, and literal or IRI object kinds. Reuse established vocabulary,
  contexts, and project naming conventions.

**LD02**: Prefer nested nodes over `@graph` or `$graph` when one root can express the
  data.

**LD03**: Prefer compact terms expanded by the context for reusable vocabulary values.

**LD04**: Prefer a registered context IRI over its local file path.

**LD05**: Give domain entities stable identifiers independent of source filenames and
  locations. A document identifier may identify the document itself. Mint an
  HTTP IRI only when its publication route is established; otherwise follow the
  project's local identifier convention.

**LD06**: Use YAML 1.2 semantics. Quote ambiguous scalars and raw JSON-LD keywords with
  double quotes; leave unambiguous scalars unquoted. Reuse context definitions
  and YAML anchors when appropriate.

**LD07**: Give resource-valued properties IRI coercion with `"@type": "@id"` when
  scalar values must expand to IRIs. Check the expanded values and datatypes.

**LD08**: sparqld applies its bundled default context before a source's own context.
  Use `$id`, `$type`, and other provided dollar aliases in graph bodies without
  adding a context URL solely for them. Keep `@context` and keywords inside
  context definitions in standard JSON-LD form. A later context can override
  or clear the default. For other consumers, declare the context they require.
