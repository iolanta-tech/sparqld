---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/index.md
"@type": schema:TechArticle
icon: material/book-open-page-variant-outline
hide: [toc]
name: sparqld command reference
title: Reference
description: Command-line arguments and options for sparqld.
---

# :material-console-line: Command reference

```console
sparqld [OPTIONS] <DIRECTORY>
```

<div class="grid cards" markdown>

-   :material-folder-outline:{ .lg .middle } **`<DIRECTORY>`**

    ---

    **Required** · Directory to serve

-   :material-ip-network-outline:{ .lg .middle } **`--host <HOST>`**

    ---

    Listening address · Default `127.0.0.1`

-   :material-ethernet:{ .lg .middle } **`--port <PORT>`**

    ---

    Listening port · Default `7737`

-   :material-eye-off-outline:{ .lg .middle } **`--no-watch`**

    ---

    Load once without watching for file changes

-   :material-filter-outline:{ .lg .middle } **`--pattern <GLOB>`**

    ---

    Include or exclude source paths; repeat for multiple patterns

-   :material-help-circle-outline:{ .lg .middle } **`-h`, `--help`**

    ---

    Show command help

-   :material-tag-outline:{ .lg .middle } **`-V`, `--version`**

    ---

    Show the installed version

</div>

## :material-tune-variant: Combine options

Options can be composed in any order:

{{ command('sparqld ./examples --host 0.0.0.0 --port 8080 --no-watch') }}

## :material-filter-outline: Select source files

`--pattern` matches paths relative to the directory being served. A normal
pattern includes matching supported RDF files. Prefix a pattern with `!` to
exclude matches. When no include patterns are supplied, every supported RDF
file remains eligible, so an exclusion alone removes paths from the default
set. A plain directory path matches that directory and everything below it.

For example, serve Turtle files while leaving an archive outside the dataset:

{{ command("sparqld ./data --pattern '**/*.ttl' --pattern '!archive'") }}

Patterns filter source files after `sparqld` recognizes their extensions. A
matching file with an unsupported format remains ignored. Excluded files do not
appear in the file catalog. A local JSON-LD context remains available to an
included source that declares it, even when the context file is excluded from
the dataset.

!!! warning

    `--host 0.0.0.0` makes the endpoint reachable from the network. `sparqld`
    provides neither authentication nor TLS, so use it only behind controls
    that restrict access to trusted clients.
