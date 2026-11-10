---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": quickstart/cli.md
"@type": schema:HowTo
icon: material/console
name: Starting in CLI
title: Starting in CLI
description: Serve Markdown-LD files and run a SPARQL query with sparqld and sq.
---

# :material-console: Starting in CLI

This guide needs Cargo. [Install Rust and Cargo](https://rustup.rs/) if it is
not already available.

Install `sparqld` with Cargo:

{{ command('cargo install sparqld') }}

Install [:fontawesome-brands-github: `ktk/sq`](https://github.com/ktk/sq) to send the query:

{{ command('cargo install --git https://github.com/ktk/sq') }}

!!! warning "`cargo install sq` installs the wrong package"

    The crates.io crate named `sq` is unrelated. Use the GitHub install above.

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

{{ result_data('orbits.tsv') }}

Each row is a body followed by what it orbits.

Queries are read-only, so exploring the dataset cannot modify your files.

## :material-refresh: See a live update

In `data/alpha-centauri.md`, change `Alpha Centauri` to a new name and save the
file. Run the same `sq` command again: the result changes without restarting
`sparqld`.

Browse the [compatible SPARQL clients](../reference/clients/index.md) for other
ways to query the endpoint, or prefer [Starting with Agents](agents.md) for the
skill-based path.
