---
name: evolve
description: Run `j evolve` and resolve the next lint finding group. Use when asked to evolve the repository, run `j evolve`, or fix the next Ruff, Flake8/WPS, Cargo formatting, or Clippy failure.
---

# Evolve

Run `j evolve` from the repository root. If `j` is unavailable, run
`.venv/bin/j evolve`. Then act on the reason it stopped. Resolve one coherent
lint problem at a time. Keep unrelated work intact and make the next failure
easier to understand.

## Act on the command result

`j evolve` stops at the first of these:

1. The working tree has uncommitted changes. Report that status and stop. Do
   not format, lint, or edit.
2. `j fmt` changed files. Report those edits and stop. Do not start a lint fix
   on top of uncommitted formatting.
3. `j lint` printed findings. Group them and fix one group, as below.

When `j evolve` exits 0, report that the tree is clean, formatted, and lint-free.
Do not invent another task.

## Choose and resolve a group

Group the printed findings by tool and diagnostic code, with the description,
count, and affected files. For WPS, use the `WPS###` code as the initial group
key; split it when its occurrences require materially different fixes.

Select the group with the safest shared, low-risk fix. Use occurrence count
only to break ties. State the selected group, its locations, and the intended
approach, then implement every safe instance of that group in the current
working tree. Do not start a second group in the same invocation.

Inspect each affected call site before editing. Preserve behavior and existing
project conventions. Leave an occurrence unresolved when it does not share the
selected safe fix, explain why, and include it in the remaining findings.

Refactor source by default. When a narrowly scoped Flake8/WPS configuration
change is a viable alternative, present the code and configuration approaches,
with consequences and a recommendation, then wait for the user's choice before
changing configuration. Likewise, pause for a short options-based interview
when the fix has a meaningful API, behavior, or design tradeoff. Otherwise,
continue directly after the proposal.

## Verify and learn

1. Run `j fmt`.
2. Run `j lint --diff`. It reports findings only on lines the working tree
   changes. Those findings must be zero: an edit in one place must not leave a
   lint failure on the lines it touches.
3. `j evolve` stops on the uncommitted fix, so do not use it for this check.
4. Run relevant existing tests when the refactor changes behavior; run
   `cargo test --locked` when Rust code changed.
5. Review the final diff for accidental formatter or unrelated changes.
6. When `j fmt` exits 0 and `j lint --diff` prints nothing, propose `/commit`.
   Do not commit until the user accepts.

When a validated choice applies across the project—for example, the same
approved pattern is used in multiple occurrences—or the user calls it a
project practice, propose a concise addition to this skill. Update this file
only after explicit approval, and keep the rule specific enough to guide a
future lint fix without duplicating general project guidance.
