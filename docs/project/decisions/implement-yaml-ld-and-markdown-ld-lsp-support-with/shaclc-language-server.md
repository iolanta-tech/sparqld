---
title: "Build on SHACLC Language Server"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on SHACLC Language Server

{{ universe_metadata('index.md', status) }}

This planning shorthand assumes that SHACLC Language Server [:fontawesome-brands-github: `jeswr/shaclc-language-server`](https://github.com/jeswr/shaclc-language-server) supplies the LSP foundation. This universe extracts or generalizes its VS Code-oriented boundary, then replaces its SHACL Compact Syntax pipeline with support for JSON-LD, YAML-LD, and Markdown-LD.

[SHACLC Language Server is available as a published VS Code Marketplace extension](https://marketplace.visualstudio.com/items?itemName=jeswr.shaclc-language-server), providing an installable starting point for assessing its server and extension boundaries.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Evaluate reusable VS Code extension and LSP boundaries, and determine whether a generic server can be extracted without retaining editor-specific assumptions.
- [ ] :material-file-code-outline: Replace the SHACL Compact Syntax parsing and analysis pipeline with JSON-LD processing while preserving protocol-facing language features.
- [ ] :material-file-tree-outline: Build a YAML-LD syntax tree and source map from semantic values and JSON-LD paths to YAML key and value spans.
- [ ] :material-language-markdown-outline: Extract Markdown-LD front matter, retain its document offset, and translate JSON-LD diagnostics and navigation ranges back into the Markdown buffer.
- [ ] :material-graph-outline: Integrate a JSON-LD processor with explicit context loading, local-IRI policy, relative-IRI resolution, and diagnostic-to-source-range translation.
- [ ] :material-link-variant: Interpret `@type: @id` coercion to classify IRI-valued values; publish document links and Go to Definition results for every matching workspace definition.
- [ ] :material-content-save-outline: Make in-memory, unsaved buffers override on-disk documents for parsing, context resolution, indexing, and navigation.
- [ ] :material-test-tube: Add protocol and end-to-end tests for diagnostics, links, relative IRIs, single and multiple definitions, and unsaved edits through a non-VS Code client.
- [ ] :material-package-variant-closed: Package a generic server and editor integrations, then document independent installation and file associations beyond the Marketplace extension.

## :material-alert-outline: Consequences

- [ ] :material-merge: Separate reusable LSP code from the SHACLC VS Code extension and its SHACL Compact Syntax assumptions.
- [ ] :material-code-braces: Maintain the replacement JSON-LD, YAML-LD, and Markdown-LD semantic pipeline as a downstream architecture.
- [ ] :material-account-wrench: Maintain compatibility tests across the Marketplace extension and at least one non-VS Code client.
- [ ] :material-package-variant-closed: Establish and maintain a generic-server distribution channel in addition to the extension package.
