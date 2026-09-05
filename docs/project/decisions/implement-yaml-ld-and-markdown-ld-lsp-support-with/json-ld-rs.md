---
title: "Build on json-ld-rs"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on json-ld-rs

{{ universe_metadata('index.md', status) }}

This planning path adopts `json-ld-rs` as the semantic foundation and builds an LSP around it. It does not adopt an existing LSP implementation; "universe" is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `timothee-haudebourg/json-ld`](https://github.com/timothee-haudebourg/json-ld) provides the Rust [`json-ld`](https://crates.io/crates/json-ld) crate for JSON-LD processing, but no language-server transport or editor integration.

## :material-format-list-checks: Implementation checklist

- [ ] Create LSP transport, lifecycle handling, document storage, position conversion, and request routing for JSON-LD, YAML-LD, and Markdown-LD.
- [ ] Build span-preserving JSON, YAML-LD, and Markdown front-matter syntax layers, with a source map from semantic value paths to original key and scalar ranges.
- [ ] Bridge mapped values to `json-ld` processing while retaining JSON Pointer or path associations for diagnostics and semantic results.
- [ ] Define a document-loader and context policy, including safe resolution of relative local IRIs without allowing workspace escape.
- [ ] Interpret [`@type: @id`](https://www.w3.org/TR/json-ld11/#type-coercion) values as IRIs, publish clickable document links, and return every matching workspace definition for navigation.
- [ ] Keep unsaved buffers authoritative and tolerate invalid text during incremental edits without discarding usable syntax or prior semantic state.
- [ ] Build a workspace symbol and reference index that reconciles changed in-memory documents with on-disk files.
- [ ] Add protocol, JSON-LD conformance, source-range, navigation, and editor end-to-end tests.
- [ ] Package a Rust binary and implement editor-distribution metadata and installation paths.

## :material-call-split: Universe consequences

- [ ] Own the LSP’s transport, incremental-analysis model, workspace indexing, editor integration, and release lifecycle in addition to YAML-LD and Markdown-LD support.
- [ ] Maintain the adapters that connect parser-specific diagnostics and JSON-LD processing to authored source ranges.
- [ ] Track `json-ld-rs` API and JSON-LD behaviour changes while preserving a stable LSP-facing semantic model.
