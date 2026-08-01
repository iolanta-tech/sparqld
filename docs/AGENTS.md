# Documentation guidance

- On every page under `docs/project/decisions/`, place
  `{{ adr_metadata(date, status) }}` immediately after the H1 so visible ADR
  metadata is derived from frontmatter.
- Keep `**/AGENTS.md` in `exclude_docs` in `mkdocs.yml` so agent guidance is not
  published as documentation.
