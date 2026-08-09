---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": quickstart.md
"@type": schema:HowTo
icon: material/rocket-launch-outline
hide: [toc]
name: Quickstart
title: Quickstart
description: Serve Markdown-LD files and run a SPARQL query with sparqld.
---

# :material-rocket-launch-outline: Quickstart

This guide needs Cargo. [Install Rust and Cargo](https://rustup.rs/) if it is
not already available.

Install `sparqld` with Cargo:

{{ command('cargo install sparqld') }}

Install [`sq`](https://github.com/ktk/sq) to send the query:

{{ command('cargo install --git https://github.com/ktk/sq') }}

## :material-server-outline: Serve a directory

Create a directory for the data:

{{ command('mkdir -p data') }}

In your editor, create `data/alpha-centauri.md` and paste this source:

{{ example_data('markdown-ld/alpha-centauri.md') }}

Start `sparqld` in this terminal:

{{ command('sparqld ./data') }}

Keep this terminal running. The endpoint is available at
`http://127.0.0.1:7737/` and watches the directory for changes.

## :material-database-search-outline: Query the endpoint

Open a second terminal. This query shows what orbits what in the example.

Save it as `orbits.rq`:

{{ source('docs/queries/orbits.rq') }}

Run it with `sq`:

{{ command('sq -e http://127.0.0.1:7737/ -f orbits.rq') }}

The query returns:

{{ result_data('orbits.tsv') }}

Each row is a body followed by what it orbits.

Queries are read-only, so exploring the dataset cannot modify your files.

## :material-refresh: See a live update

In `data/alpha-centauri.md`, change `Alpha Centauri` to a new name and save the
file. Run the same `sq` command again: the result changes without restarting
`sparqld`.

Next, see [Agents](reference/agents.md) or browse the
[compatible SPARQL clients](reference/clients/index.md).
