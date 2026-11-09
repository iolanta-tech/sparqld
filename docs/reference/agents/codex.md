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
hide: [toc]
---

# :fontawesome-brands-openai: Codex

Install the skill for Codex:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g -a codex
```

Prefer that path. The Codex marketplace plugin from
[:fontawesome-brands-github: `iolanta-tech/sparqld`](https://github.com/iolanta-tech/sparqld)
packages the same skill when you install through Codex instead:

```sh
codex plugin marketplace add iolanta-tech/sparqld --ref main
codex plugin add sparqld@sparqld-development
```

Ask Codex to set up the knowledge base:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

Approve installing `sparqld` and registering the MCP server when it asks. Open a
new thread after registration. Use the `query` tool from that server.

With the example dataset configured, this Codex session used its query tool to
find the recorded planets:

{{ agent_conversation('alpha-centauri') }}
