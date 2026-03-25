---
name: foundry:done
description: Checkpoint Foundry in git and/or collapse a finished spec into a completed-work summary plus archived files.
---

# foundry:done

Use after a meaningful chunk of work, or when a spec is **fully complete** and should become history.

## Guardrails

- **Git sync** commits whatever changed under `~/.foundry/`; use an accurate message.
- **Collapse** is irreversible as an *active* spec: the CLI moves working files into `completed/<spec-id>/archive/`. Only run when the user agrees the spec is finished.
- The **summary narrative is agent-authored**. Foundry creates `summary.md` (empty) on `collapse prepare`; you fill it before `finalize`.

## Steps

### A. Ongoing checkpoint (git backup)

1. Summarize what changed (completed tasks, decisions, follow-ups).
2. If `foundry git init` was run for `~/.foundry/`:

   ```bash
   foundry git sync "<short accurate message>"
   ```

3. If git backup is not enabled, skip—files still live under `~/.foundry/`.

### B. Finish a spec (completed-work record)

1. Confirm all tasks are checked:

   ```bash
   foundry --json spec status <project> <spec-id-or-partial>
   ```

   Expect derived phase / state indicating **completed_pending_collapse** before collapsing.

2. Get collapse guidance (JSON only):

   ```bash
   foundry --json spec instructions collapse <project> <spec-id-or-partial>
   ```

   Follow `instruction` and `needs_prepare`.

3. Create the completed directory scaffold:

   ```bash
   foundry spec collapse prepare <project> <spec-id-or-partial>
   ```

4. **Write** `summary.md` at the path implied by `~/.foundry/<project>/completed/<spec-id>/summary.md` (or from the prepare JSON when using `--json`). Include what shipped, scope, and key decisions.

5. Finalize (moves `spec.md`, `task-list.md`, `notes.md`, `meta.json` into `archive/`):

   ```bash
   foundry spec collapse finalize <project> <spec-id-or-partial> --confirm
   ```

6. Optionally run **A** again to commit the collapsed layout.
