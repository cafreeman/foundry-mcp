---
name: foundry:done
description: Checkpoint Foundry changes into the optional ~/.foundry git backup.
---

# foundry:done

Use this after you have finished a meaningful chunk of work on Foundry files.

## Steps

1. Write a short, accurate summary of what changed (what you completed, what moved, what decisions were made).
2. If `foundry git init` has been run for `~/.foundry/`, sync it:

```bash
foundry git sync "<your summary message>"
```

3. If git backup is not enabled, skip the command; Foundry files still live under `~/.foundry/`.
