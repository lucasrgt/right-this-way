<h1 align="center">Right This Way</h1>

<p align="center"><strong>Proven repository patterns for coding agents.</strong></p>

<p align="center">
  <a href="https://github.com/lucasrgt/right-this-way/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/lucasrgt/right-this-way/ci.yml?branch=main&style=flat-square" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="MIT License"></a>
  <a href="https://github.com/lucasrgt/right-this-way/releases"><img src="https://img.shields.io/github/v/release/lucasrgt/right-this-way?style=flat-square" alt="Release"></a>
</p>

Coding agents can copy syntax. They struggle to preserve a repository's real
implementation patterns across distant features, long tasks, different agents,
and context compaction.

Right This Way records a proven pattern once, retrieves it before analogous
work, and audits the final diff for concrete deviations.

```text
A proven pattern exists -> rtw add
A task is starting      -> rtw guide
Work is finishing       -> rtw check
```

<table>
<tr><td><b>Repository-native</b></td><td>Ways are readable TOML files versioned with the codebase and reviewed by the whole team.</td></tr>
<tr><td><b>Reference-backed</b></td><td>Every way points to tracked files that already implement the pattern successfully.</td></tr>
<tr><td><b>Cross-directory recall</b></td><td>Scopes, tags, and FTS5 find a view-model pattern even when the next view model lives in another feature tree.</td></tr>
<tr><td><b>Bounded context</b></td><td>Agents receive only the most relevant ways and inspect their references instead of loading the full corpus.</td></tr>
<tr><td><b>Fail-closed audit</b></td><td>A two-stage isolated judge must confirm the same way, changed path, and line.</td></tr>
<tr><td><b>Agent-independent</b></td><td>Use the native CLI, the embedded skill, or local stdio MCP from Codex, Claude, Hermes, Pi, and other harnesses.</td></tr>
</table>

---

## Quick install with your agent

Copy this block directly into your coding agent:

```text
Install Right This Way from https://github.com/lucasrgt/right-this-way.
Use the latest GitHub release for this operating system. In the repository,
run:

rtw init --agent-file AGENTS.md

If CLAUDE.md or another agent instruction file is also tracked, pass one
--agent-file option for each file. Read .rtw/SKILL.md, verify that
.rtw/ways/ is versioned while .rtw/config.local.toml and .rtw/index.sqlite
are ignored, then run
rtw guide --task "Adopt Right This Way" --path AGENTS.md.

Do not invent initial ways. Add a way only when a reusable pattern is already
backed by tracked repository files.
```

---

## Installation

### Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/lucasrgt/right-this-way/main/scripts/install.sh | sh
```

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/lucasrgt/right-this-way/main/scripts/install.ps1 | iex
```

### Build from source

```bash
cargo install --git https://github.com/lucasrgt/right-this-way --locked
```

RTW is one native binary. It requires no hosted service, daemon, Node runtime,
Python runtime, or project language integration.

---

## Quick start

Initialize a Git repository:

```bash
rtw init --agent-file AGENTS.md --agent-file CLAUDE.md
```

Record an existing feature view-model pattern:

```bash
rtw add \
  --title "Feature view models" \
  --intent "Create a view model for a feature workflow" \
  --guidance "Keep state transitions in the view model and expose immutable state to the view." \
  --scope "src/features/**" \
  --tag "view-model" \
  --tag "state-management" \
  --reference "src/features/orders/order_view_model.dart"
```

Guide a new feature before editing:

```bash
rtw guide \
  --task "Create the payment feature view model" \
  --path "src/features/payments/payment_view_model.dart"
```

Audit the final uncommitted diff:

```bash
rtw check --task "Create the payment feature view model"
```

Audit committed work before a pull request or push:

```bash
rtw check --base main --task "Review the payment feature"
```

| Exit code | Meaning |
| --- | --- |
| `0` | Aligned with the relevant ways |
| `1` | Confirmed pattern deviations were found |
| `2` | The audit could not complete |

---

## The way model

RTW has one durable concept:

> A way is a reusable implementation pattern backed by tracked repository code.

```toml
schema = 1
id = "01k..."
title = "Feature view models"
intent = "Create a view model for a feature workflow"
guidance = "Keep state transitions in the view model and expose immutable state to the view."
scopes = ["src/features/**"]
tags = ["state-management", "view-model"]
references = ["src/features/orders/order_view_model.dart"]
recorded_at = "2026-07-26T12:00:00Z"
recorded_by = "Ana Developer"
recorded_commit = "9e8d..."
```

A useful way answers four questions:

| Question | Field |
| --- | --- |
| When should this pattern apply? | `intent` |
| What structure or invariants matter? | `guidance` |
| Where is analogous work likely to happen? | `scopes` and `tags` |
| Which real implementation proves it? | `references` |

Do not record generic best practices, temporary experiments, preferences, or
hypothetical architecture. If the repository does not already contain a
proven reference, it is not a way yet.

---

## Retrieval

`rtw guide` combines:

| Signal | Strength | Example |
| --- | --- | --- |
| Scope | Highest | `src/features/**` matches the new payment feature |
| Tag | High | `view-model` crosses unrelated feature directories |
| Full text | Supporting | Task language matches title, intent, or guidance |

The default result limit is eight. `rtw check` independently retrieves at most
12 ways from the actual changed paths. The full corpus never needs to enter the
agent's context. The disposable SQLite index is reused while its corpus
fingerprint matches the versioned TOML files. Direct edits, Git updates,
missing indexes, and corrupt indexes trigger an automatic transactional
rebuild.

---

## Semantic alignment check

Deterministic retrieval decides which ways are relevant. Semantic alignment
still depends on the configured judge model because repository patterns cannot
be reduced to one universal linter.

RTW limits that judgment:

1. only changed Git paths and diff lines are supplied;
2. only retrieved, versioned ways may be used;
3. the first pass proposes concrete deviations;
4. a second isolated pass must confirm the same way, path, and line;
5. invented evidence or judge failure exits with code 2.

RTW improves the evidence and protocol around the model. It does not claim that
an LLM can never miss a subtle deviation.

---

## Configuration

`rtw init` creates an ignored developer-local configuration at
`.rtw/config.local.toml`:

```toml
schema = 1

[judge]
command = ["codex", "exec", "--skip-git-repo-check", "--sandbox", "read-only", "-"]
```

Configuration precedence is:

| Scope | Location |
| --- | --- |
| Developer-local | `.rtw/config.local.toml` |
| Optional team override | `.rtw/config.toml` |
| Linux and macOS user | `~/.config/right-this-way/config.toml` |
| Windows user | `%APPDATA%\right-this-way\config.toml` |

This lets one teammate use Codex and another use Claude without conflicts.
Commit `.rtw/config.toml` only when the team intentionally standardizes the
judge command.

The judge command must read the audit prompt from standard input and write only
the requested JSON object to standard output.

---

## MCP

Start the local stdio server:

```bash
rtw mcp
```

Generic MCP host configuration:

```json
{
  "mcpServers": {
    "right-this-way": {
      "command": "rtw",
      "args": ["mcp"]
    }
  }
}
```

| Tool | Purpose |
| --- | --- |
| `rtw_guide` | Retrieve relevant ways |
| `rtw_add` | Record a proven way |
| `rtw_check` | Audit a Git diff |

Every MCP tool requires an explicit repository root and calls the same core as
the CLI.

---

## RTW, NYA, and AVP

The projects are independent and solve different memory problems:

| Project | Durable question |
| --- | --- |
| Right This Way | How does this repository already implement this kind of thing correctly? |
| [Not You Again](https://github.com/lucasrgt/not-you-again) | Which corrected failure must not recur? |
| AVP | Which acceptance behavior must hold? |

RTW is positive precedent. NYA is corrected failure memory. AVP is executable
acceptance. They can complement one another without sharing storage or runtime
dependencies.

---

## Benchmarks

RTW ships reproducible, machine-readable benchmark protocols and retains the
agent events, exact diffs, candidate hashes, and deterministic evaluations.

| Protocol | Result | Key evidence |
| --- | --- | --- |
| [Paired coding agent](benchmarks/results/v0.1.1-paired-gpt-5.3-codex-spark/REPORT.md) | PASS | 5/5 RTW arms passed, every guide and check was observed, zero regressions |
| [1,024-way stress](benchmarks/results/v0.1.1-stress-1024/REPORT.md) | PASS | 128/128 targets ranked first, warm recall p95 0.125 s |
| [10,000-way stress](benchmarks/results/v0.1.1-stress-10000/REPORT.md) | PASS | 64/64 targets ranked first, warm recall p95 0.594 s |

The paired baselines also passed all five small synthetic tasks, so they are
reported as passing ties and RTW claims zero measured improvements in that run.
The first 10,000-way attempt exposed an early global FTS cutoff that placed one
target second. Version 0.1.1 removes that cutoff and adds a permanent
more-than-64-ties regression test.

See [the benchmark protocols](benchmarks/README.md) for the outcome taxonomy,
primary sources, reproduction commands, limitations, and complete results.

---

## Repository layout

```text
right-this-way/
├── src/                         Rust CLI, MCP, storage, retrieval, audit
├── tests/                       Unlimited black-box and protocol tests
├── xtask/                       Canonical repository quality gate
├── benchmarks/                  Paired agent and large-corpus protocols
├── assets/right-this-way/       Portable agent skill
├── scripts/                     Release installers
└── .github/workflows/           CI and release packaging
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full runtime design.

---

## Build and contribute

```bash
cargo build --release
cargo install cargo-llvm-cov tokei --locked
cargo xtask verify
```

`cargo xtask verify` is the canonical gate for local work, coding agents, CI,
and releases.

| Invariant | Gate |
| --- | --- |
| Maintained production code | At most 500 lines |
| Shared runtime line coverage | At least 95 percent without rounding |
| Packaged entrypoint | End-to-end binary smoke |
| Product model | One way across three daily operations |
| Storage | Versioned TOML plus disposable SQLite |
| Failure behavior | Judge and protocol failures fail closed |
| Transport | CLI and MCP call the same core |

---

## License

Right This Way is available under the [MIT License](LICENSE).
