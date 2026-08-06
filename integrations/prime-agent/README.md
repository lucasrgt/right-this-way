# Right This Way for Prime Agent

This optional capability package is a thin adapter around the standalone `rtw`
Rust CLI. It adds bounded automatic `guide`, explicit operator commands, and a
conditional model skill without reading semantic records or reimplementing
Right This Way behavior.

## Install

Install `rtw` on `PATH`, then run:

```bash
prime-agent package install /absolute/path/to/right-this-way/integrations/prime-agent
```

Use `/reload` in a live Prime session. Set `RTW_BIN` or pass
`--rtw-bin /absolute/path/to/rtw` when needed.

## Activation and precedence

The package activates only when the Git root contains `.rtw/SKILL.md`. It is
fully suppressed when `<git-root>/csm.toml` exists, even if the standalone marker
also remains. CSM then owns Prime retrieval and verification; direct standalone
CLI use remains available. In inactive repositories the package invokes no
`rtw` process, exposes no command or skill, and paints no status.

## Surface

- ``/rtw guide <task>` and `/rtw check [--base=REF] <task>``
- `/rtw status`
- `/rtw auto guide on|off`

Automatic `guide` is enabled by default and can be disabled at launch with
`--rtw-auto-guide off`. Checks are always explicit. The adapter exposes no
repository adoption or semantic-record mutation command.

Every process uses a literal argv array, the resolved Git root as cwd, a
configurable timeout, cancellation, control-sequence sanitization, and a 64 KiB
UTF-8 output cap. Nonzero exits, cancellation, and truncation remain visible.
Repository output is delimited as lower-priority project knowledge.
