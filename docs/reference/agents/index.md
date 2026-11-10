---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/index.md
"@type": schema:TechArticle
icon: material/robot-outline
name: Agents
title: Agents
description: Let agents query a local sparqld knowledge base.
hide: [toc]
---

# :material-robot-outline: Agents

Install the `sparqld` skill, then ask the agent to set up the knowledge base:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g
```

```text
/sparqld set up a queryable knowledge base at my documents folder
```

<div class="grid cards" markdown>

-   [:simple-claude: **Claude Code**](claude-code.md)

    ---

    Install the skill for Claude Code and register `sparqld mcp`.

-   [:simple-cursor: **Cursor**](cursor.md)

    ---

    Install the skill for Cursor and approve its MCP config edit.

-   [:fontawesome-brands-openai: **Codex**](codex.md)

    ---

    Install the skill for Codex, or use the marketplace plugin instead.

-   [:material-console: **OpenCode**](opencode.md)

    ---

    Install the skill for OpenCode and register `sparqld mcp`.

-   [:simple-modelcontextprotocol: **MCP server**](mcp.md)

    ---

    Tools, prompts, and result shapes for `sparqld mcp`.

</div>

Agents without MCP can use the endpoint URL with a SPARQL HTTP client, such as
[`sq`](../clients/sq/index.md).
