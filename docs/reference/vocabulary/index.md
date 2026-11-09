---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/vocabulary/index.md
"@type": schema:TechArticle
icon: material/book-alphabet
hide:
  - toc
title: Vocabulary
description: RDF terms used for context registration and file catalog metadata.
---

# :material-book-alphabet: Vocabulary

These RDF terms describe context registrations and
[file catalog](../file-catalog.md) metadata.

<div class="grid cards" markdown>

-   :material-link-variant:{ .lg .middle } **[`jsonld:context`](http://www.w3.org/ns/json-ld#context)**

    ---

    Link a source file to the context URIs requested while parsing it.

-   :material-code-json:{ .lg .middle } **[`jsonld:Context`](http://www.w3.org/ns/json-ld#Context)**

    ---

    Mark a node as a saved context. With [`schema:contentUrl`](http://schema.org/contentUrl), the graph that contains the node registers that URL.

-   :material-web:{ .lg .middle } **[`schema:contentUrl`](http://schema.org/contentUrl)**

    ---

    Give the remote URL of a saved context. The graph that contains the node is the local body.

</div>
