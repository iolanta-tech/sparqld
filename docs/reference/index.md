---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/index.md
"@type": schema:TechArticle
icon: material/console-line
name: sparqld command reference
title: Reference
description: Command-line arguments and options for sparqld.
---

# :material-console-line: Command reference

```console
sparqld [SHARED OPTIONS] http [HTTP OPTIONS]
```

Place `--directory`, `--config`, `--pattern` & `--query-timeout` before
`http` or `mcp`. Place `--host`, `--port`, `--watch` & `--no-watch` after `http`.

<div class="grid cards" markdown>

-   :material-folder-outline:{ .lg .middle } **`-d`, `--directory <DIRECTORY>`**

    ---

    Directory to serve · Required unless set in the [configuration file](#configuration-file)

-   :material-file-cog-outline:{ .lg .middle } **`-c`, `--config <PATH>`**

    ---
    Select a [configuration file](#configuration-file)

-   :material-ip-network-outline:{ .lg .middle } **`--host <HOST>`**

    ---

    Listening address · Default `127.0.0.1`

-   :material-ethernet:{ .lg .middle } **`--port <PORT>`**

    ---

    Listening port · Default `7737`

-   :material-eye-outline:{ .lg .middle } **`--watch`, `--no-watch`**

    ---

    Enable or disable watching for file changes · Default enabled

-   :material-filter-outline:{ .lg .middle } **`--pattern <GLOB>`**

    ---

    Include or exclude source paths; repeat for multiple patterns

-   :material-timer-outline:{ .lg .middle } **`--query-timeout <SECONDS>`**

    ---

    Stop queries that run longer than this many seconds · Default unlimited

-   :material-help-circle-outline:{ .lg .middle } **`-h`, `--help`**

    ---

    Show command help

-   :material-tag-outline:{ .lg .middle } **`-V`, `--version`**

    ---

    Show the installed version

</div>

## :simple-modelcontextprotocol: MCP server

```console
sparqld [--directory <DIRECTORY>] [--config <PATH>] [--pattern <GLOB>]... [--query-timeout <SECONDS>] mcp
```

`mcp` exposes the directory to a [Model Context Protocol client](agents/mcp.md)
over standard input & output. It always watches for source changes.

## :material-file-cog-outline: Configuration file

Both server modes look for `sparqld.toml` in the working directory at startup.
Pass `--config <PATH>` to use a different file without loading `sparqld.toml`
from the working directory. The specified file must exist.
A relative `directory` setting resolves from the directory containing the
configuration file. A relative `--directory` value resolves from the working
directory.

For example, create the file below in a directory with a `data/`
subdirectory that contains your Turtle files. Run `sparqld http` from the directory
that contains the file to start the HTTP server on port `8080`.

{{ example_data('configuration/sparqld.toml', title='Configuration') }}

Values you supply on the command line override the corresponding file settings.
Repeat `--pattern` to supply a complete replacement for `patterns`.
On the HTTP server, use either `--watch` or `--no-watch` to override `http.watch`.

## :material-tune-variant: Combine options

To serve a directory on port `8080` and load it once:

{{ command('sparqld --directory ./examples http --host 0.0.0.0 --port 8080 --no-watch') }}

## :material-timer-outline: Limit query time

`--query-timeout` and `query-timeout` take a positive whole number of seconds.
Leaving both unset keeps query execution unlimited.

The budget covers parsing, evaluation, reading every result, and building the
response. It excludes waiting for the first dataset, reading the request, and
sending the response. Each query has its own deadline. Partial results are
discarded.

Cancellation is cooperative: sorting, aggregation, and similar work between
engine checkpoints can run past the deadline. `sparqld` does not stop the
process.

{{ command('sparqld --directory ./examples --query-timeout 20 http') }}

## :material-clock-start: Wait for the first dataset

The listener binds while `sparqld` loads its first dataset. Send an ordinary
query to `/` with a client-side timeout; it waits until that dataset is ready.
The `Serving … at …` log line remains useful for people, but is not a client
readiness contract.

## :material-filter-outline: Pattern matching

`--pattern` matches paths relative to the directory being served. A normal
pattern includes matching supported RDF files. Prefix a pattern with `!` to
exclude matches. When no include patterns are supplied, every supported RDF
file remains eligible, so an exclusion alone removes paths from the default
set. A plain directory path matches that directory and everything below it.

For example, serve Turtle files while leaving an archive outside the dataset:

{{ command("sparqld --directory ./data --pattern '**/*.ttl' --pattern '!archive' http") }}

Patterns filter source files after `sparqld` recognizes their extensions. A
matching file with an unsupported format remains ignored. Excluded files do not
appear in the file catalog. A local JSON-LD context remains available to an
included source that declares it, even when the context file is excluded from
the dataset.

!!! warning

    `--host 0.0.0.0` makes the endpoint reachable from the network. `sparqld`
    provides neither authentication nor TLS, so use it only behind controls
    that restrict access to trusted clients.
