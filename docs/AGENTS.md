# Documentation guidance

## Home persona

Write Home pages for a prospective adopter deciding whether `sparqld` fits a
local RDF project and seeking a first successful query. They can use a command
line but need not know RDF, JSON-LD, or the repository's development process.
State the outcome and constraints, give the smallest path to success, and link
to Reference for operating contracts or Project for contribution.

Reference and Project define their own personas in their scoped `AGENTS.md`
files. In a Quickstart, optimize for the first successful query: treat the
example data as copy-pasteable input and defer syntax mechanics to the
reference unless they are needed to complete the task.

Explain sparqld-specific behavior and less familiar RDF concepts where they
first become necessary.

## Review

After any change under `docs/` or to `mkdocs.yml`—including a one-character
edit—run the mkdocs skill review before claiming the work done. When planning
changes under `docs/` or to `mkdocs.yml`, list that review as an explicit plan
step.

## Prose

- Use the default graph for ordinary query examples. Select a source graph when
  graph selection is the behavior being demonstrated.
- Prefer existing DBpedia URIs for linked-data examples when a suitable entry
  exists. For queries over the shared default graph, choose subjects and facts
  that distinguish each example's results while keeping related subjects
  connected to the page's topic. Do not use source-graph restrictions only to
  separate one example from another.

- State an absence only when it corrects a likely reader expectation and
  materially affects use or understanding; otherwise describe the supported
  behavior.

When explanatory prose or a table uses an RDF QName, make the QName a link to
the IRI of the term it denotes.

- On an ADR index page under `docs/project/decisions/`, place
  `{{ adr_metadata(date, status) }}` immediately after the H1 so visible ADR
  metadata is derived from frontmatter. Supported statuses include `draft`,
  `undecided`, `decided`, and `living` (chosen outcome kept; comparison may
  gain rows when new alternatives appear). Supporting alternative-universe
  pages use `{{ universe_metadata(adr_path, status) }}` instead: they identify
  their parent ADR and their chosen, excluded, or undecided status rather than
  presenting themselves as ADRs.
- Keep `**/AGENTS.md` in `exclude_docs` in `mkdocs.yml` so agent guidance is not
  published as documentation.
