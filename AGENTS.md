# sparqld guidance

## Documentation sources

- Keep code, query, result, and data examples in their dedicated source files.
  Render them in documentation through the corresponding MkDocs macro instead
  of embedding their contents directly in a page.

## Commands

- The `jeeves` package runs with the direnv virtualenv active. Invoke `ruff`, `flake8`,
  `mkdocs`, and Cargo through `sh` (`sh.ruff`, `sh.flake8`, `sh.mkdocs`,
  `sh.cargo`).
- Bake optional or repeated arguments with `bake`. Pass rustfmt `--check` after
  `--` on `sh.cargo.fmt`.

## JSON documents

- Parse a nested JSON document with a Pydantic model and read the fields from
  that model.

## Browser validation

- Before starting a development server, check whether the project's documented
  development URL is responsive in the browser. Reuse it when available; start
  a server only when it is unavailable.

## Subdirectory guidance

- [docs/AGENTS.md](docs/AGENTS.md) - MkDocs authoring and ADR metadata conventions
