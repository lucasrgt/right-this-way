---
name: rtw
description: Use standalone Right This Way repository knowledge before editing and its explicit semantic gate before completion.
---

# Right This Way

This skill is available only because the Git root contains `.rtw/SKILL.md`
and does not contain `csm.toml`. If CSM is adopted, use only the CSM integration;
do not invoke the standalone adapter and duplicate retrieval or checks.

Before editing, retrieve relevant proven ways:

```bash
"${RTW_BIN:-rtw}" guide --task="<goal>" --path <expected-path>
```

The Prime extension injects guidance automatically when enabled. Inspect referenced tracked files rather than copying blindly.

Before completion, run:

```bash
"${RTW_BIN:-rtw}" check --task="<completed work>" --base HEAD
```

Exit code 1 means repository findings remain; fix or report them and rerun. Exit
code 2 or a killed, failed, or truncated provider means the operation did not
complete and must never be reported as a pass.

Never run `rtw init` or `rtw add` unless the user explicitly requests repository adoption or recording an already-proven pattern.
