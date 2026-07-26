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
