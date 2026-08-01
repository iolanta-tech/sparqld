---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/index.md
"@type": schema:TechArticle
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

-   :material-file-cog-outline:{ .lg .middle } **`--config <FILE>`**

    ---

    Reserved for a TOML configuration file

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
