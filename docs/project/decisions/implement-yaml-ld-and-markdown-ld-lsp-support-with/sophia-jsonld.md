---
title: "Build on Sophia JSON-LD"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on Sophia JSON-LD

{{ universe_metadata('index.md', status) }}

This planning path uses Sophia JSON-LD as the semantic foundation for a new language server, rather than as an LSP implementation. “Universe” is planning shorthand for the consequences of committing to this foundation.

Sophia JSON-LD [:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs/tree/main/jsonld) is the `sophia_jsonld` parser-and-serializer package in Sophia [:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs); its manifest [:fontawesome-brands-github: `Cargo.toml`](https://github.com/pchampin/sophia_rs/blob/main/jsonld/Cargo.toml) depends on [`json-ld` 0.15.1](https://crates.io/crates/json-ld/0.15.1) and offers optional `file:` and HTTP(S) context retrieval.

## :material-format-list-checks: Implementation checklist

- [ ] Create the Rust LSP transport, lifecycle, document model, workspace store, and request routing that Sophia does not provide.
- [ ] Parse JSON-LD, YAML-LD, and Markdown-LD into one document model while retaining a source map from JSON-LD value paths to JSON, YAML, or embedded-front-matter spans.
- [ ] Design and implement a mapped-input adapter from each document model to Sophia JSON-LD parsing and serialization APIs [:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs/tree/main/jsonld), retaining enough semantic provenance to return LSP ranges.
- [ ] Configure a context and document-loader policy around Sophia’s optional file and HTTP context features [:fontawesome-brands-github: `Cargo.toml`](https://github.com/pchampin/sophia_rs/blob/main/jsonld/Cargo.toml), including local-document access, permitted remote access, and relative-IRI base resolution.
- [ ] Interpret JSON-LD [type coercion to `@id`](https://www.w3.org/TR/json-ld11/#type-coercion) so IRI-valued strings are distinguished from literals and surfaced as clickable LSP [document links](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentLink).
- [ ] Build an IRI definition index and serve every matching workspace target through the LSP [definition response](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_definition).
- [ ] Make semantic conversion, source maps, document links, and the definition index update from authoritative unsaved text on [open and full-document change notifications](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didChange).
- [ ] Translate parser, context, and semantic errors back through the source map to exact JSON, YAML, or Markdown diagnostic ranges.
- [ ] Add JSON-LD conformance fixtures and LSP end-to-end tests for diagnostics, `@type: @id` links, relative cross-file resolution, Markdown offsets, unsaved buffers, and multiple definition targets.
- [ ] Package the Rust binary and provide editor integration, installation, versioning, and release automation.

## :material-call-split: Universe consequences

- [ ] Own the full LSP implementation and its editor-facing compatibility surface.
- [ ] Maintain adapters and source maps across three authored syntaxes while following Sophia and its [`json-ld`](https://crates.io/crates/json-ld) dependency releases.
- [ ] Establish and maintain the server’s context-loading, workspace-indexing, and navigation policies as product behaviour.
