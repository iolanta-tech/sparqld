---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": vocabulary/index.md
"@type": schema:TechArticle
icon: material/book-alphabet
hide:
  - toc
title: Vocabulary
description: RDF terms used by sparqld in the file catalog.
---

# :material-book-alphabet: Vocabulary

The [file catalog](../reference/file-catalog.md) uses these RDF terms.

<div class="grid cards" markdown>

-   :material-link-variant:{ .lg .middle } **[`jsonld:context`](http://www.w3.org/ns/json-ld#context)**

    ---

    Link a source file to the context URIs requested while parsing it.

-   :material-code-braces:{ .lg .middle } **[`jsonld:Context`](http://www.w3.org/ns/json-ld#Context)**

    ---

    Type a resource that registers an absolute context URI.

-   :material-file-link:{ .lg .middle } **[`schema:contentUrl`](https://schema.org/contentUrl)**

    ---

    Point a `jsonld:Context` registration at its local body file.

</div>
