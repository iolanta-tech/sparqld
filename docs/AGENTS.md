# Documentation guidance

## Reader

Write for a developer with a version-controlled, file-based knowledge base who
wants immediate, safe SPARQL access for themselves and their agents, without
operating a database or duplicating the data.

Assume comfort with command-line tools, but not prior exposure to RDF or
JSON-LD syntax. In a Quickstart, optimize for the first successful query:
treat the example data as copy-pasteable input and defer syntax mechanics to
the reference unless they are needed to complete the task.

Explain sparqld-specific behavior and less familiar RDF concepts where they
first become necessary.

## Prose

- State an absence only when it corrects a likely reader expectation and
  materially affects use or understanding; otherwise describe the supported
  behavior.

When explanatory prose or a table uses an RDF QName, make the QName a link to
the IRI of the term it denotes.

- On every page under `docs/project/decisions/`, place
  `{{ adr_metadata(date, status) }}` immediately after the H1 so visible ADR
  metadata is derived from frontmatter.
- Keep `**/AGENTS.md` in `exclude_docs` in `mkdocs.yml` so agent guidance is not
  published as documentation.
