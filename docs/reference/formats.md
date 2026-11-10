---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/formats.md
"@type": schema:TechArticle
hide: [toc]
name: Supported file formats
title: File formats
description: File extensions recognized by sparqld and their support status.
---

# :material-file-multiple-outline: File formats

`sparqld` selects a parser from a file's extension. Every source enters its own
named graph and shares the same watching and error-reporting behavior. Dataset
formats keep their declared named graphs; see [Named graphs](named-graphs.md).

<div class="grid cards" markdown>

-   :material-code-json:{ .lg .middle } **[JSON-LD](https://www.w3.org/TR/json-ld11/)**

    ---

    `.json`, `.jsonld`

    Linked data in JSON. Declare terms in the document's `@context`; the
    default context already supplies the dollar keyword aliases.

-   :simple-yaml:{ .lg .middle } **[YAML-LD](https://www.w3.org/TR/yaml-ld-10/)**

    ---

    `.yamlld`

    JSON-LD authored as YAML 1.2.

-   :material-language-markdown:{ .lg .middle } **[Markdown-LD](https://spec.commonmark.org/)**

    ---

    `.md`

    YAML-LD front matter. Any YAML front matter is treated as linked data; the
    Markdown body is ignored, and Markdown without YAML front matter is
    ignored entirely.

-   :material-format-list-bulleted:{ .lg .middle } **[Notation3](https://www.w3.org/TeamSubmission/n3/)**

    ---

    `.n3`

    Compact RDF graph syntax with additional Notation3 features.

-   :material-view-grid-outline:{ .lg .middle } **[N-Quads](https://www.w3.org/TR/n-quads/)**

    ---

    `.nq`

    Line-oriented RDF dataset syntax.

-   :material-format-list-bulleted:{ .lg .middle } **[N-Triples](https://www.w3.org/TR/n-triples/)**

    ---

    `.nt`, `.txt`

    Line-oriented RDF graph syntax.

-   :material-xml:{ .lg .middle } **[RDF/XML](https://www.w3.org/TR/rdf-syntax-grammar/)**

    ---

    `.rdf`, `.xml`

    XML serialization of an RDF graph.

-   :material-graph-outline:{ .lg .middle } **[TriG](https://www.w3.org/TR/trig/)**

    ---

    `.trig`

    Turtle syntax for RDF datasets.

-   :material-turtle:{ .lg .middle } **[Turtle](https://www.w3.org/TR/turtle/)**

    ---

    `.ttl`

    Compact RDF graph syntax.

</div>

## :material-code-braces-box: Contexts

JSON-LD, YAML-LD, and Markdown-LD start from the
[default context](contexts/default-context.md) and then use the contexts their
documents declare. See [Contexts](contexts/index.md).
