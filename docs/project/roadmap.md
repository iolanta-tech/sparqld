---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
  datePublished: schema:datePublished
"@id": project/roadmap.md
"@type": schema:TechArticle
name: sparqld roadmap
title: Roadmap
description: Work identified by evaluating the sparqld website as its intended reader.
datePublished: 2026-08-03
---

# :material-map-marker-path: Roadmap

This roadmap records work identified by evaluating the published site for a developer with a
version-controlled, file-based knowledge base who wants immediate, safe SPARQL
access for themselves and their agents, without operating a database or
duplicating the data.

The documentation built successfully in strict mode. All 13 published routes
returned HTTP 200, with no browser-console warnings or horizontal overflow at
desktop and mobile sizes. The issues below concern the adoption journey,
behavioral contract, and differences between the documentation and software.

## :material-information-outline: Medium

- [ ] **Preserve named graphs inside dataset files.** TriG and N-Quads files
   currently reject named graphs. Load each embedded graph under an IRI composed
   from the file graph IRI and the embedded graph name, separated by `#`, and
   record its relationship to the file graph in `sparqld:`. Choose and document
   the relationship term as part of this work.

- [ ] **Add opt-in OWL 2 RL reasoning.** Add a `--reasoning` flag that uses
   `reasonable` to materialize its supported OWL 2 RL inferences into the
   dedicated `<reasoning:inferred>` named graph. Keep asserted source graphs
   unchanged and reasoning disabled by default. Re-materialize the derived
   graph atomically whenever the asserted dataset changes.
