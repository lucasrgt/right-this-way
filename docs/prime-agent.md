# Prime Agent integration

The optional package at `integrations/prime-agent` wraps the standalone `rtw`
CLI without reading `.rtw` records or reproducing Rust semantics.

Install it after placing `rtw` on `PATH`:

```bash
prime-agent package install /absolute/path/to/right-this-way/integrations/prime-agent
```

Run `/reload` in an active Prime session. The adapter activates only when the Git
root contains `.rtw/SKILL.md`. A root `csm.toml` always suppresses it;
CSM then owns Prime retrieval and checks while the standalone CLI stays usable.

The adapter exposes `/rtw status`, `/rtw guide`, explicit `/rtw check`,
and a session-only `/rtw auto guide on|off` toggle. Automatic
`guide` defaults to on and can be disabled at launch with
`--rtw-auto-guide off`. It never exposes repository adoption or semantic
record mutation commands.

All subprocesses use literal argv, the Git root as cwd, cancellation, a timeout,
and a 64 KiB UTF-8 output cap. Nonzero exits, killed processes, and truncation
remain explicit. Injected output is delimited as repository knowledge rather
than higher-priority instructions.
