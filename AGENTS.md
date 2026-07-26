# Right This Way Engineering Guide

All repository artifacts must be written in English.

## Product contract

Right This Way exposes one public concept, the way, and three daily operations:

1. `rtw add`
2. `rtw guide`
3. `rtw check`

`rtw init` installs repository assets. `rtw mcp` exposes the same three
operations to MCP hosts.

A way must describe a reusable implementation pattern already proven by
tracked repository code. Do not introduce generic memories, rules, scars,
violations, templates, candidates, or graduation states.

## Engineering constitution

1. Production code under `src/` must remain at or below 500 code lines as
   measured by `tokei`.
2. Shared runtime line coverage must remain at or above 95 percent without
   rounding. The seven-line process entrypoint is verified by an end-to-end
   packaged-binary smoke test.
3. Test code is unlimited and must live under `tests/`.
4. Production behavior may not be moved into scripts, generated files,
   integrations, or test helpers to evade the line budget.
5. Git is the durable source of truth for ways.
6. SQLite is a disposable projection with no unique knowledge.
7. CLI and MCP must call the same core operations.
8. Judge and protocol failures must fail closed.

## Change discipline

Prefer the smallest complete implementation. Add a dependency or abstraction
only when it removes more maintained behavior than it introduces.

Before reporting implementation work complete, run `cargo xtask verify`. This
is the canonical local, CI, and release gate.
<!-- rtw:instructions:start -->
## Right This Way

This repository uses Right This Way (`rtw`) to preserve proven implementation patterns across agents and sessions.

1. At task start, run `rtw guide --task "<goal>" --path <expected-path>` before editing. Read every returned way and inspect its referenced files.
2. Rerun `rtw guide` when scope changes, context is reset or compacted, or work moves into an unfamiliar area.
3. Follow the invariants and structure of relevant ways. Adapt names and domain details instead of copying code blindly.
4. Use `rtw add` only for a pattern already proven in tracked repository code and useful for future work. Every way requires reusable scopes, tags, guidance, and at least one tracked reference.
5. Before committing an uncommitted diff, run `rtw check --task "<completed task>"`.
6. For committed review, pull-request preparation, or pre-push review, run `rtw check --base <target-revision> --task "<review context>"`.
7. Rerun the applicable check after changing the reviewed diff. Exit code 1 requires alignment and another check. Exit code 2 is an incomplete audit and must never be reported as a pass.

Tests and linters do not replace `rtw check`. Do not report work ready until the applicable check exits with code 0.
<!-- rtw:instructions:end -->
