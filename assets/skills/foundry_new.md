---
name: foundry:new
description: Scaffold a new Foundry spec directory and then populate it via normal file edits.
---

# foundry:new

Use this when starting a new feature spec.

## Steps

1. Ensure the repo is linked (`.foundry` symlink) or know the `<project>` name under `~/.foundry/`.
2. Create the spec scaffold:

```bash
foundry spec init <project> <feature>
```

`<feature>` must be kebab-case (example: `auth-flow`).

3. Open the new directory under `.foundry/specs/<timestamp>_<feature>/` (or the printed path) and edit:

- `spec.md`
- `task-list.md`
- `notes.md`

Foundry intentionally does not accept content on the CLI; you write the markdown yourself.
