---
title: "Build on rdflangserver"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on rdflangserver

{{ universe_metadata('index.md', status) }}

This path is unavailable as a PyPI dependency today, but it records what would change if rdflangserver became publishable. It would use rdflangserver as the Python LSP foundation and generalize its RDF language support for JSON-LD, YAML-LD, and Markdown-LD.

[:fontawesome-brands-github: `niklasl/rdflangserver`](https://github.com/niklasl/rdflangserver) declares repository version `0.1.0-dev` [:fontawesome-brands-github: `pyproject.toml`](https://github.com/niklasl/rdflangserver/blob/main/pyproject.toml); the corresponding [`rdflangserver` PyPI package](https://pypi.org/project/rdflangserver/) is unavailable.

## :material-format-list-checks: Implementation checklist

- [ ] :material-package-variant-closed: Publish and release a stable rdflangserver package with a versioned compatibility policy before making it a dependency.
- [ ] :material-lan-connect: Audit the PyGLS and RDFLib boundaries for LSP lifecycle, document storage, URI handling, position conversion, diagnostics, and workspace synchronization.
- [ ] :material-file-code-outline: Add JSON-LD, YAML-LD, and Markdown-LD syntax layers that retain mappings from semantic-value paths to authored key and scalar spans.
- [ ] :material-code-json: Connect a JSON-LD processor to the mapped semantic representation without losing the source locations needed for diagnostics and navigation.
- [ ] :material-book-open-variant: Define context loading and local-IRI policy, including relative resolution, permitted local files, document bases, and remote-context handling.
- [ ] :material-link-variant: Recognize [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion), expose IRI-valued scalars as clickable links, and return every matching workspace definition.
- [ ] :material-content-save-outline: Make unsaved buffers authoritative for parsing, context resolution, diagnostics, document links, and the definition index.
- [ ] :material-file-search-outline: Maintain an incremental workspace index of IRI definitions and references across JSON-LD, YAML-LD, and Markdown-LD.
- [ ] :material-test-tube: Add processor, source-map, and end-to-end LSP tests for source ranges, context failures, IRI links, relative cross-file navigation, multiple definitions, and unsaved edits.
- [ ] :material-package-variant-closed: Publish the generic server and document editor integrations and file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-rocket-launch-outline: Adoption cannot begin until a supported package is released; the project must either sponsor that release or carry a forked distribution.
- [ ] :material-language-python: The implementation, dependency management, and editor integration follow the Python and PyGLS ecosystems.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD require a maintained semantic-value-to-source-span bridge because JSON-LD processors do not preserve their authored syntax locations.
- [ ] :material-account-wrench: The project must track upstream changes in rdflangserver, PyGLS, RDFLib, and the selected JSON-LD processor behind a stable LSP-facing model.
