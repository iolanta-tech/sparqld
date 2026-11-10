---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": index.md
"@type": schema:SoftwareApplication
icon: material/folder-search-outline
name: sparqld
title: sparqld
description: A live, read-only SPARQL endpoint for linked data stored in files.
---

# :material-folder-search-outline: SPARQL for RDF files

![sparqld logo: a linked-data graph inside a folder](images/logo-cropped.png)

`sparqld` turns a directory of RDF files into a live, read-only SPARQL
endpoint. Keep your data in version control and query its current state without
importing it into a database.

[![CI](https://github.com/iolanta-tech/sparqld/actions/workflows/ci.yml/badge.svg)](https://github.com/iolanta-tech/sparqld/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/sparqld.svg)](https://crates.io/crates/sparqld)
[![License](https://img.shields.io/crates/l/sparqld.svg)](https://github.com/iolanta-tech/sparqld#license)

<div class="grid cards" markdown>

-   :material-folder-multiple:{ .lg .middle } **Files stay authoritative**

    ---

    Use Markdown-LD, YAML-LD, JSON-LD, and the rest of your repository as the
    source of truth.

-   :material-sync:{ .lg .middle } **Queries stay current**

    ---

    `sparqld` watches the directory and makes each successful edit available
    atomically.

-   :material-shield-lock-outline:{ .lg .middle } **Safe for people and agents**

    ---

    The endpoint is permanently read-only: queries explore the knowledge base
    without changing its files.

</div>

## Get started

<div class="start-paths" markdown>

<div class="start-path" markdown>

### :material-robot-outline: Agents

Install the `sparqld` skill for your coding agent:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g
```

Then ask:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

[:material-rocket-launch-outline: Starting with Agents](quickstart/agents.md){ .md-button .md-button--primary }

</div>

<p class="start-paths__or">or</p>

<div class="start-path" markdown>

### :material-server-outline: Local endpoint

<div class="install-command" markdown>

{{ command('cargo install sparqld') }}

{{ command('sparqld ./data') }}

</div>

The directory is served at `http://127.0.0.1:7737/`. Query it from your shell or
application; edit the files to change the data.

[:material-rocket-launch-outline: Starting in CLI](quickstart/cli.md){ .md-button .md-button--primary }
[:material-console-line: Reference](reference/index.md){ .md-button }

</div>

</div>

No database service, import script, or synchronization layer is required.
