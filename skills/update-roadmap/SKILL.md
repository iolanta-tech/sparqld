---
name: update-roadmap
description: >-
  Review the project as the documented intended reader, propose Roadmap items,
  ask which to apply, then edit docs/project/roadmap.md. Use when the user asks
  to update the roadmap, refresh roadmap findings, or run /update-roadmap.
---

# Update Roadmap

Read-only review first; edit `docs/project/roadmap.md` only after the user
chooses which proposed items to apply.

## Persona

Adopt the intended reader from [docs/AGENTS.md](../../docs/AGENTS.md):

> A developer with a version-controlled, file-based knowledge base who wants
> immediate, safe SPARQL access for themselves and their agents, without
> operating a database or duplicating the data.
>
> Comfortable with command-line tools, but not prior exposure to RDF or JSON-LD
> syntax. In a Quickstart, optimize for the first successful query.

Also read [AGENTS.md](../../AGENTS.md) and the current
[docs/project/roadmap.md](../../docs/project/roadmap.md).

## Workflow

### 1. Subagent persona review

Spawn a **read-only** subagent. Do not let it edit files, commit, or push.

Instruct it to:

1. Use the persona above.
2. Apply defect-first review rules: discrete, actionable issues that affect
   correctness, security, performance, maintainability, or documentation clarity
   for that persona; prefer issues introduced by the current branch and working
   tree over ancient pre-existing nits; do not invent findings.
3. Inspect commits vs the default base (`origin/main` when available), the
   working tree, and the published-site journey (install → Quickstart → query →
   reference → project docs).
4. Return findings first, ordered by severity:

   `[P1] Imperative title — path:line`

   with one short paragraph each (scenario + why wrong), then a brief overall
   assessment and material test gaps / residual risks.

Priorities: `P0` release blocker, `P1` urgent, `P2` ordinary, `P3` low-impact
still worth fixing. If none: `No findings.`

### 2. Propose Roadmap changes

From the subagent findings (and any clearly related persona friction you confirm
in the tree), draft **candidate Roadmap checklist items**. Do not edit the
Roadmap yet.

Rules for each candidate:

- Match existing Roadmap style: bold short title, then one or two sentences of
  concrete work.
- Map severity to sections: `P0`/`P1` → High, `P2` → Medium, `P3` → Low.
- Skip duplicates of items already present on the Roadmap (same intent).
- Skip speculative product ideas that are not grounded in the review.
- Prefer site/docs/pluglet/install friction visible to the persona.

Present to the user:

1. A short review summary (or “No findings”).
2. A **numbered list** of proposed Roadmap additions, each showing target
   section (High / Medium / Low) and the full checklist text that would be
   inserted.

### 3. Ask which changes to apply

Ask which numbered proposals to apply. Prefer a multi-select question when the
runtime supports it; otherwise ask for numbers (e.g. `1, 3`).

Do **not** edit the Roadmap until the user answers. If they choose none, stop.

### 4. Apply selected changes

Edit only [docs/project/roadmap.md](../../docs/project/roadmap.md):

1. Insert each selected item under the matching High / Medium / Low section as
   an unchecked `- [ ]` entry.
2. Keep existing items; do not reorder unrelated entries unless needed for a
   coherent High→Low severity grouping of the new ones.
3. Set `datePublished` in the frontmatter to today’s date (`YYYY-MM-DD`).
4. Do not commit unless the user asks.

Report what was added and where.

## Out of scope

- Implementing the Roadmap work items themselves.
- Editing ADRs, code, or docs beyond `docs/project/roadmap.md` in step 4.
- Publishing releases or opening PRs unless the user separately asks.
