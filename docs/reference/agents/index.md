---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/index.md
"@type": schema:TechArticle
name: Agents
title: Agents
description: Let agents query a local sparqld knowledge base.
---

# :material-robot-outline: Agents

`sparqld` gives agents read-only SPARQL access to your knowledge base.

<div class="grid cards" markdown>

-   [:simple-modelcontextprotocol: **MCP server**](mcp.md)

    Run `sparqld mcp <DIRECTORY>` to connect an MCP-capable agent.

-   [:material-robot-outline: **Other agents**](https://github.com/iolanta-tech/sparqld/tree/main/skills/sparqld)

    From a full checkout, give an MCP-capable agent `skills/sparqld`.

-   [:fontawesome-brands-openai: **Codex**](codex.md)

    Configure the `sparqld` Codex plugin and its local MCP server.

</div>

Agents without MCP can use the endpoint URL with a SPARQL HTTP client, such as
[`sq`](../clients/sq/index.md).
