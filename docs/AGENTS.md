# Documentation guidance

## Reader

Write for a developer with a version-controlled, file-based knowledge base who
wants immediate, safe SPARQL access for themselves and their agents, without
operating a database or duplicating the data.

Assume comfort with command-line tools and basic linked-data terminology.
Explain sparqld-specific behavior and less familiar RDF concepts where they
first become necessary.

- On every page under `docs/project/decisions/`, place
  `{{ adr_metadata(date, status) }}` immediately after the H1 so visible ADR
  metadata is derived from frontmatter.
- Keep `**/AGENTS.md` in `exclude_docs` in `mkdocs.yml` so agent guidance is not
  published as documentation.
