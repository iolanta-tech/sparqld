---
title: "Build on Qlue-ls"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on Qlue-ls

{{ universe_metadata('index.md', status) }}

This planning shorthand assumes that Qlue-ls [:fontawesome-brands-github: `IoannisNezis/Qlue-ls`](https://github.com/IoannisNezis/Qlue-ls) supplies the LSP foundation. The resulting server replaces or generalizes its SPARQL-specific language intelligence for JSON-LD, YAML-LD, and Markdown-LD.

[Qlue-ls is a Rust SPARQL language-server library with native and WASM server entry points](https://docs.rs/qlue-ls/latest/qlue_ls/); its documented distribution also supports an installed command and editor clients.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Map Qlue-ls’s server lifecycle, document store, URI handling, position conversion, capability registration, and diagnostics interfaces; isolate components that can remain language-neutral.
- [ ] :material-file-code-outline: Replace or generalize the SPARQL parser, formatter, completion, and backend-query pipeline so JSON-LD documents become first-class inputs without retaining SPARQL assumptions.
- [ ] :material-file-tree-outline: Build a YAML-LD syntax tree and source map that map semantic values and JSON-LD paths to YAML key and value spans.
- [ ] :material-language-markdown-outline: Extract Markdown-LD front matter, retain its document offset, and translate JSON-LD diagnostics and navigation ranges back into the Markdown buffer.
- [ ] :material-graph-outline: Select and integrate a JSON-LD processor, including context loading, relative-IRI resolution, local-document policy, and error translation to source ranges.
- [ ] :material-link-variant: Interpret `@type: @id` coercion to classify IRI-valued values; publish document links and Go to Definition results for every matching workspace definition.
- [ ] :material-content-save-outline: Make in-memory, unsaved buffers override on-disk documents for parsing, context resolution, indexing, and navigation.
- [ ] :material-test-tube: Establish JSON-LD, YAML-LD, and Markdown-LD conformance fixtures plus end-to-end LSP tests for diagnostics, links, single and multiple definitions, relative IRIs, and unsaved edits.
- [ ] :material-package-variant-closed: Package the resulting server, document editor configuration and file associations, and verify native and WASM/editor integration paths.

## :material-alert-outline: Consequences

- [ ] :material-merge: Decide whether Qlue-ls’s SPARQL capabilities remain in the same distributable or are separated before semantic-web features and configuration are generalized.
- [ ] :material-code-braces: Audit every reused protocol and workspace component for coupling to SPARQL AST types, endpoint-driven completion, and query-specific configuration.
- [ ] :material-account-wrench: Carry the maintenance cost of a downstream fork or coordinate upstream abstractions before relying on Qlue-ls as the common LSP base.
- [ ] :material-web: Verify that the required YAML-LD and Markdown-LD processing, filesystem access, and local-document policies work consistently in both Qlue-ls native and WASM targets.
