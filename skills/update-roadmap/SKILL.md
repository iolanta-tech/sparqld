---
name: update-roadmap
description: >-
  Review sparqld as its documented intended reader and update its mission-led
  Mermaid roadmap. Use when the user asks to update or refresh the roadmap, or
  runs /update-roadmap.
---

# Update Roadmap

Review first. Edit `docs/project/roadmap.md` only after the user chooses which
proposed outcomes to apply.

## Read first

Read [AGENTS.md](../../AGENTS.md), [docs/AGENTS.md](../../docs/AGENTS.md), and
the current [roadmap](../../docs/project/roadmap.md).

## Review

Spawn a read-only subagent. It must not edit files, commit, or push. Ask it to:

1. Apply the Roadmap's intended audience and defect-first review rules: report discrete,
   evidenced problems affecting correctness, security, performance,
   maintainability, or documentation clarity. Do not invent findings.
2. Inspect commits against the default base (`origin/main` when available),
   the working tree, and the published-site journey: install → Quickstart →
   query → reference → project docs.
3. Return findings first, ordered by severity, in the form
   `[P1] Imperative title — path:line`, with a brief scenario and impact. Then
   report the overall assessment and material test gaps or residual risks.

Use `P0` for a release blocker, `P1` for urgent work, `P2` for ordinary work,
and `P3` for a lower-impact issue. Report `No findings.` when appropriate.
Treat the working-tree roadmap as authoritative. Use the default base only to
identify regressions; do not turn a pre-existing roadmap format or an
uncommitted user edit into a finding.

## Propose outcomes

Use the findings and confirmed related friction to propose candidate roadmap
outcomes. Do not edit the roadmap yet.

Each candidate must:

- name a finite, observable outcome for a person or agent;
- have a clear completion test in its wording;
- be grounded in review evidence; and
- be distinct from an existing outcome.

State outcomes, not implementation activities or undefined quality claims.
For example, write “Users can query embedded graph IRIs with fragments without
graph-identity collisions,” not “Support fragments” or “Correct named-graph
querying.” Do not turn severity, priorities, release numbers, or maintenance
labels into roadmap goals.

Identify a dependency only when the evidence establishes it. In the graph,
`A --> B` means that completing B is blocked by A. Do not manufacture
dependencies merely to make the graph look structured.

Present a short review summary, then a numbered list of candidate outcomes with
their evidence and any proposed dependency arrows. Ask the user which outcomes
to apply. Do not edit if they select none.

## Apply selected outcomes

Edit only [docs/project/roadmap.md](../../docs/project/roadmap.md):

1. Preserve the existing project mission. If it is absent, ask the user to
   provide one; do not invent it from a product backlog.
2. Maintain a single `flowchart LR` Mermaid diagram immediately after the
   roadmap's one-sentence arrow explanation. Use rounded nodes (`goal("…")`),
   make the rightmost mission node begin with `<strong>Mission</strong>`, and
   point arrows from blockers on the left toward outcomes they enable.
3. Add each selected outcome as its own rounded node. Connect it directly to
   the mission unless a real intermediate dependency is evidenced. The mission
   is an enduring direction, not a completion criterion.
4. Remove completed, superseded, or duplicate outcome nodes when the evidence
   supports doing so. Do not add a checklist, priority sections, synthetic
   aggregate nodes, or explanatory implementation backlog beneath the diagram.
5. Update `datePublished` to today (`YYYY-MM-DD`). Preserve unrelated
   frontmatter and page structure.
6. Do not commit unless the user asks.

After editing, build the documentation and validate the rendered roadmap in
Chromium through Playwright. Confirm that Mermaid rendered, every node is a
rounded rectangle, the mission is rightmost, and the page has no horizontal
overflow at desktop width.

## Out of scope

- Implementing roadmap outcomes.
- Editing ADRs, code, or documentation outside `docs/project/roadmap.md`.
- Publishing releases or opening pull requests.
