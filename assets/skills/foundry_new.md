---
name: foundry:new
description: Scaffold a new Foundry spec and start the workflow (empty files, then agent-authored markdown).
---

# foundry:new

Use when starting a new feature or change as a Foundry spec.

## Guardrails

- Foundry **does not** accept spec body text on the CLI. You create scaffolds, then **edit markdown** yourself.
- `<feature>` must be **kebab-case** (e.g. `auth-flow`).

## Steps

1. Ensure the project exists and the repo is linked when possible:

   ```bash
   foundry project init <project>    # if needed
   foundry link [project]            # in the repo root, optional name if auto-detect works
   ```

2. Create the spec scaffold:

   ```bash
   foundry spec init <project> <feature>
   ```

   Init prints JSON (`path` field); parse it or read the path from stdout when wrapping this command.

3. **Immediately** open the new directory (under `.foundry/specs/<timestamp>_<feature>/` when linked) and edit:

   - `spec.md` — intent, scope, non-goals
   - `task-list.md` — markdown checkboxes (`- [ ]` / `- [x]`) for actionable work
   - `notes.md` — scratch space

4. Check workflow state:

   ```bash
   foundry spec status <project> <spec-id-or-partial>
   ```

   Follow the `instruction` field until tasks exist and you are ready to implement.

5. Do not mark the spec “done” in prose only—use **task-list checkboxes** so `spec status` and `spec instructions apply` stay accurate.
