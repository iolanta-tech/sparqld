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

This roadmap records work identified by evaluating the published site for a
developer with a version-controlled, file-based knowledge base who wants
immediate, safe SPARQL access for themselves and their agents, without
operating a database or duplicating the data.

## :material-alert-outline: High

- [ ] **Support embedded graph IRIs with fragments.** Scope an embedded graph
   using a valid encoded mapping of its source and original IRI, including
   fragment identifiers. Apply that mapping consistently to rewritten graph
   references and catalog entries, and add a fragment-bearing dataset fixture.

## :material-information-outline: Medium

- [ ] **Maintain an exclude list.** Add a command-line option for paths that
   `sparqld` must not watch or load, and exclude `.git` by default. This avoids
   processing transient repository files such as Git's lock files.

- [ ] **Add opt-in OWL 2 RL reasoning.** Add a `--reasoning` flag that uses
   `reasonable` to materialize its supported OWL 2 RL inferences into the
   dedicated `<reasoning:inferred>` named graph. Keep asserted source graphs
   unchanged and reasoning disabled by default. Re-materialize the derived
   graph atomically whenever the asserted dataset changes.

- [ ] **Add `sparqld.toml` configuration.** Define a predictable configuration
   location or explicit path, and precedence relative to command-line options.
   Use it for persistent settings such as the remote-host whitelist and future
   reasoning configuration, while keeping command-line invocation simple.

- [ ] **Add opt-in remote Linked Data ingestion.** When an HTTP(S) IRI from a
   configured host whitelist appears in the asserted graph, resolve it as
   Linked Data and download the result into a dedicated local directory. Load
   that copy into the dataset so users can ingest resources from services such
   as DBpedia, Wikidata, and nanopublication servers. Decide whether the
   resolver belongs in the core or a plugin before implementation.

- [ ] **Stream query responses.** Serialize bindings and graph results directly
   into the HTTP response instead of buffering the complete result in memory.
   This keeps broad queries available without imposing an artificial result
   limit.

## :material-information-outline: Low

- [ ] **Publish a minimal SPARQL Service Description.** Describe the root
   endpoint and its query capability so standards-aware clients and agents can
   discover it without application-specific source configuration.
