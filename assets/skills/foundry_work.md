---
name: foundry:work
description: Drive implementation from a Foundry spec using JSON instructions (task progress, context files, next steps).
---

# foundry:work

Use when the user wants to **implement** work tracked in a Foundry spec (`task-list.md` checkboxes).

## Guardrails

- Always start from **JSON** so you respect `state` and do not guess paths.
- If `state` is **blocked**, improve `spec.md` / `task-list.md` before coding.
- If `state` is **all_done**, switch to **foundry:done** (collapse + git) instead of inventing new tasks silently.
- Keep edits scoped to the **next unchecked** task unless the user expands scope.

## Steps

1. Resolve the spec if needed:

   ```bash
   foundry --json list specs <project>
   ```

2. Fetch apply instructions:

   ```bash
   foundry --json spec instructions apply <project> <spec-id-or-partial>
   ```

3. Parse the payload:

   - `state` — `blocked` | `in_progress` | `all_done`
   - `progress` — total / complete / remaining
   - `tasks` — checkbox-aligned rows with `done` flags
   - `context_files` — absolute paths to open
   - `instruction` — what to do **right now**

4. Read every file in `context_files` before editing code.

5. Implement the **next** incomplete task. Prefer the smallest change that satisfies that task.

6. Update `task-list.md`: flip `- [ ]` → `- [x]` for work you actually finished.

7. Re-run step 2 until `state` is `all_done` or you hit ambiguity—then pause and ask the user.
