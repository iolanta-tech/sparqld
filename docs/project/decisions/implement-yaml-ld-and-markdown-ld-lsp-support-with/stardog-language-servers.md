---
title: "Build on Stardog Language Servers"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on Stardog Language Servers

{{ universe_metadata('index.md', status) }}

Planning shorthand for the world in which Stardog Language Servers [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers) is the LSP base. This page records the work needed to make that base serve generic YAML-LD and Markdown-LD, not a decision to do so.

The upstream project is a TypeScript Lerna and Yarn-workspaces monorepo [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers#developingcontributing) whose documented servers support multiple LSP transports and diagnostics, hovers, and completion [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers#features).

## :material-format-list-checks: Implementation checklist

- [ ] Map the repository's transport, document lifecycle, workspace, and request-dispatch APIs; separate reusable LSP infrastructure from Stardog-language [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers#stardog-language-servers)-specific semantics.
- [ ] Establish a TypeScript runtime, build, test, and published-package path for a generic standalone language server, independent of Stardog Studio and its extensions.
- [ ] Add a YAML-LD parser that retains source ranges and maps YAML nodes to JSON-LD values without round-tripping through generated JSON text.
- [ ] Add Markdown-LD extraction for frontmatter, translating YAML ranges back to Markdown-document offsets.
- [ ] Integrate a JSON-LD processor with an explicit context-loading policy and a local-IRI resolver shared by JSON-LD, YAML-LD, and Markdown-LD documents.
- [ ] Interpret `@type: @id` coercion to classify values as IRIs or literals; expose IRI values as document links and return every matching workspace definition.
- [ ] Make incremental analysis use unsaved document buffers rather than only files on disk.
- [ ] Create fixtures and end-to-end LSP tests for diagnostics, source ranges, local and relative IRIs, multiple definitions, document links, and unsaved edits.
- [ ] Package, version, and release the generic server; verify editor integration through the documented stdio installation path [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers#integrating-with-other-editors).

## :material-alert-outline: Consequences

- [ ] Maintain a TypeScript toolchain and release process alongside any Rust-based linked-data consumers.
- [ ] Audit and either isolate or replace Stardog-specific parsing and semantic dependencies before publishing a generic server.
- [ ] Own source-map correctness across YAML frontmatter and full Markdown files, including edits that move frontmatter offsets.
- [ ] Define and test the security and caching behavior of JSON-LD context loading and local IRI resolution.
