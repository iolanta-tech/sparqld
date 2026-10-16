---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/codex.md
"@type": schema:TechArticle
name: Codex agents
title: Codex
description: Query a local sparqld knowledge base from Codex.
---

# :fontawesome-brands-openai: Codex

Install the Codex plugin from the [:fontawesome-brands-github: `iolanta-tech/sparqld`](https://github.com/iolanta-tech/sparqld)
GitHub marketplace:

```sh
codex plugin marketplace add iolanta-tech/sparqld --ref main
codex plugin add sparqld@sparqld-development
```

Start a new Codex thread and invoke `@sparqld`. Give it the absolute path to
your knowledge base. When it asks, approve the local MCP configuration. Codex
gives each knowledge base a separate server name, such as `sparqld-iolanta`.

Open another new thread after registration. Use the `query` tool from that
server to run read-only SPARQL; edit source files to change data.

With the example dataset configured through the installed plugin, this Codex
session used its query tool to find the recorded planets and moons:

{{ agent_conversation('plugin-alpha-centauri') }}
