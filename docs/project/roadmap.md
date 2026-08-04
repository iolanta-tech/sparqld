---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
  datePublished: schema:datePublished
"@id": project/roadmap.md
"@type": schema:TechArticle
name: sparqld roadmap
title: Roadmap
description: Work identified by evaluating the sparqld website as its intended reader.
datePublished: 2026-08-03
---

# :material-map-marker-path: Roadmap

This roadmap records work identified by evaluating the published site for a developer with a
version-controlled, file-based knowledge base who wants immediate, safe SPARQL
access for themselves and their agents, without operating a database or
duplicating the data.

The documentation built successfully in strict mode. All 13 published routes
returned HTTP 200, with no browser-console warnings or horizontal overflow at
desktop and mobile sizes. The issues below concern the adoption journey,
behavioral contract, and differences between the documentation and software.

## :material-alert-octagon-outline: Critical

- [ ] **The installation command does not work.** The Quickstart presents
   `cargo install sparqld`, but `sparqld` is not published on crates.io. No
   Git-based installation or downloadable binary is offered.

- [ ] **The Quickstart's example files are unavailable to the reader.** It
   displays files from `docs/examples` but instructs the reader to run
   `sparqld ./examples`. Neither `./examples` nor `./queries` exists at the
   repository root, and an installed user would have neither directory. There
   is no clone, download, or file-creation step.

- [ ] **The Quickstart introduces an uninstalled second dependency.** The first
   query requires `sq`, but the site provides no installation instructions for
   it. Linking to its repository does not make the first-run sequence
   executable.

- [ ] **Pointing sparqld at a normal repository can produce enormous amounts of
   accidental data and errors.** The loader recursively traverses every
   directory without ignore rules and treats overloaded extensions such as
   `.json`, `.txt`, `.xml`, and `.md` as RDF candidates. Serving the sparqld
   repository root loaded 1,272 source files, failed on 1,698, and examined
   more than 40,000 others, including `.venv` and `node_modules`. Nothing on
   the site warns users to isolate data or explains exclusions.

## :material-alert-outline: High

- [ ] **The documented default-graph semantics are incorrect.** The site says
   queries without `GRAPH` see the union of source graphs. The union also
   includes the reserved file-catalog graph. The example dataset contains 12
   source triples and 12 catalog triples; an unrestricted default-graph count
   returns 24. The cheat sheet reinforces the incorrect description while its
   graph-count result visibly includes `sparqld:`.

- [ ] **The format page does not match implemented behavior.** Turtle, TriG,
   N-Triples, N-Quads, and RDF/XML are labelled "in development," although the
   loader already recognizes them and tests exercise successful Turtle
   loading. Conversely, the implementation also recognizes undocumented
   `.json`, `.txt`, and `.n3` inputs. The site does not distinguish
   "implemented but unsupported" from "not implemented."

- [ ] **The site's strongest value proposition is largely absent.** The
   Quickstart says "ordinary directory" and "live endpoint," but never clearly
   states the central benefit: files remain the source of truth, with no
   database, import, or synchronization copy. That language exists in the
   README but not the published site.

- [ ] **The agent story is demonstrative rather than operational.** "Ask your
   agent" implies that an agent can discover and query the endpoint
   automatically. The site never explains what capability the agent needs, how
   to give it endpoint access, or what command or tool it should invoke. The
   dedicated Agents page only repeats the same conversation.

- [ ] **The public-network example lacks a security warning.** The command
   reference demonstrates `--host 0.0.0.0`, but the server has no
   authentication or TLS. "Read-only" prevents SPARQL Update; it does not
   prevent anyone with network access from reading the knowledge base or
   running expensive queries.

- [ ] **`--config` is an exposed, silently ignored option.** The site calls it
    "reserved," while CLI help says it reads a TOML file. The implementation
    parses the path and discards it without checking whether the file exists. A
    user can reasonably believe their configuration was applied.

- [ ] **The failure behavior behind "changes appear automatically" is
    missing.** When an edited source becomes invalid, sparqld removes that
    source's existing graph instead of continuing to serve its previous valid
    version. It also records an `rlog:Entry` in the catalog. Both behaviors are
    important to someone relying on safe live updates, but neither is
    explained on the published site.

- [ ] **Adoption-critical project information is missing.** The site does not
    disclose that the software is under development, identify a release or
    supported Rust version, link prominently to the repository and issue
    tracker, or state a license. The repository currently has no license file
    and no Cargo license metadata.

## :material-information-outline: Medium

- [ ] **Add opt-in OWL 2 RL reasoning.** Add a `--reasoning` flag that uses
   `reasonable` to materialize its supported OWL 2 RL inferences into the
   dedicated `<reasoning:inferred>` named graph. Keep asserted source graphs
   unchanged and reasoning disabled by default. Re-materialize the derived
   graph atomically whenever the asserted dataset changes.

- [ ] **"Using the file formats that suit you" overpromises.** The wording
    suggests broad compatibility before the reader discovers that commonly
    used RDF formats are marked unsupported. This is especially jarring for a
    linked-data practitioner whose repository likely uses Turtle.

- [ ] **Markdown-LD behavior is underspecified.** The site does not explain that
    only YAML frontmatter is parsed, that the Markdown body contributes no RDF,
    that Markdown without frontmatter is ignored, or how ordinary non-RDF
    frontmatter behaves. This matters for the explicitly targeted
    documentation-repository use case.

- [ ] **The "live" behavior is asserted but never demonstrated.** The
    Quickstart starts the watcher and runs one query, but never asks the reader
    to edit a file and rerun it. The feature that differentiates sparqld
    therefore produces no visible first-run payoff.

- [ ] **The JavaScript example's runtime constraints are unclear.** It is a
    Node.js example, but the page labels it only "JavaScript." Browser code
    would also encounter the endpoint's lack of CORS headers. Readers may
    incorrectly infer that the example applies to browser applications.

- [ ] **The top-level Project section prioritizes an internal ADR over adoption
    information.** The only project content is a large language-selection
    decision. For the intended reader, status, roadmap, source repository,
    license, limitations, and contribution path would be more immediately
    useful.
