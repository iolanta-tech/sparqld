---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents.md
"@type": schema:TechArticle
name: Agents
title: Agents
description: Let agents query a local sparqld knowledge base.
---

# :material-robot-outline: Agents

`sparqld` gives agents a read-only query surface over the files in your
knowledge base. Give the agent the endpoint URL and a way to send SPARQL HTTP
requests, such as the [`sq` client](clients/sq/index.md). To give Codex the
local query workflow, install the skill from the GitHub repository:

```sh
npx skills add https://github.com/iolanta-tech/sparqld --skill sparqld --agent codex
```

With the example dataset running, ask your agent:

{{ agent_conversation('alpha-centauri') }}

{{ query_data('alpha-centauri-planets.rq', title='Query used') }}

{{ result_data('alpha-centauri-planets.tsv') }}
