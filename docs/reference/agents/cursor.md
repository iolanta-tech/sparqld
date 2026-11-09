---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/cursor.md
"@type": schema:TechArticle
name: Cursor agents
title: Cursor
description: Query a local sparqld knowledge base from Cursor.
hide: [toc]
---

# :simple-cursor: Cursor

Install the skill for Cursor:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g -a cursor
```

Ask Cursor to set up the knowledge base:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

Approve installing `sparqld` and editing the MCP config when it asks. Reload MCP
or start a new Agent chat if the tools do not appear yet. Use the `query` tool
once it is available.
