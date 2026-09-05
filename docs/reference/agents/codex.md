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

When Codex asks, approve MCP configuration and provide the absolute path to
your knowledge base. Open a new Codex thread afterwards. In that thread,
`sparqld.query` runs read-only SPARQL queries; edit source files to change data.

With the example dataset configured through the installed plugin, this Codex
session used `sparqld.query` to find the recorded planets and moons:

{{ agent_conversation('plugin-alpha-centauri') }}
