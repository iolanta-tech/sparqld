---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": index.md
"@type": schema:SoftwareApplication
hide: [toc]
name: sparqld
title: Quickstart
description: A live, read-only SPARQL endpoint for linked data stored in files.
---

# :material-rocket-launch-outline: Quickstart

![sparqld logo: a linked-data graph inside a folder](images/logo-cropped.png)

<div class="grid cards" markdown>

-   :material-folder-multiple:{ .lg .middle } **Your knowledge base**

    ---

    Keep linked data in an ordinary directory, using the file formats that suit you.

-   :material-graph-outline:{ .lg .middle } **A live SPARQL endpoint**

    ---

    Run `sparqld` to make it queryable by people and agents. Changes appear automatically. ✨

</div>

## :material-download-outline: Installation <small markdown>:simple-rust: with `cargo`</small>

<div class="install-command" markdown>

{{ command('cargo install sparqld') }}

</div>

## :material-console: Run your first query

### :material-server-outline: Serve a directory

Start with a directory:

{{ directory_tree('docs/examples') }}

Point `sparqld` at it:

{{ command('sparqld ./examples') }}

The endpoint is available at `http://127.0.0.1:7737/` and watches the directory
for changes. Use `--host` or `--port` to change the listening address, or
`--no-watch` to load once.

### :material-database-search-outline: Query the endpoint

Use any
[compatible SPARQL client](reference/clients/index.md). This query finds names
together with their source graphs:

{{ query_data('names.rq') }}

Run it with :material-database-search: [`sq`](https://github.com/ktk/sq):

{{ command('sq -e http://127.0.0.1:7737/ < queries/names.rq') }}

The live endpoint returns results from YAML-LD, Markdown-LD, and JSON-LD:

{{ result_data('names.tsv') }}

Queries are read-only, so exploring the dataset cannot modify your files.

## :material-robot-outline: Ask your knowledge base

!!! question ":material-account-outline: You"

    Which constellation contains Alpha Centauri, and what does our knowledge
    base say about it?

!!! example ":material-robot-outline: Agent"

    I’ll query the live endpoint.

    {{ query_data('alpha-centauri.rq', indent=4) }}

    {{ result_data('alpha-centauri.tsv', indent=4) }}

!!! success ":material-robot-happy-outline: Agent"

    Alpha Centauri is in the **Centaurus** constellation. The knowledge base
    describes it as the closest star system to the Solar System.
