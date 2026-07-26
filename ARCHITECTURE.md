# Right This Way Architecture

## Purpose

Right This Way is repository-local positive memory for coding agents. It
preserves how a codebase already implements recurring structures so a future
agent can reproduce the same shape in another feature, directory, language, or
session.

## Public model

The system contains one durable concept:

> A way is a reusable implementation pattern backed by tracked repository code.

| Field | Purpose |
| --- | --- |
| `intent` | When the pattern should be used |
| `guidance` | The structure and invariants to preserve |
| `scopes` | Repository paths where it is likely to apply |
| `tags` | Semantic retrieval across unrelated directories |
| `references` | Proven files the agent must inspect |
| provenance | Who recorded it and from which commit |

There are three daily operations:

```text
A proven pattern exists -> rtw add
A task is starting      -> rtw guide
Work is finishing       -> rtw check
```

## Storage

```text
.rtw/
├── config.local.toml
├── config.toml
├── SKILL.md
├── ways/
│   └── <ulid>.toml
└── index.sqlite
```

TOML way files are versioned and authoritative. SQLite FTS5 is rebuilt from
those files and may be deleted at any time. `config.local.toml` is ignored and
stores a developer's judge command. `config.toml` is an optional team override.

## Retrieval

`rtw guide` combines three independent signals:

1. glob scope matches against expected paths;
2. exact semantic tag matches against the task and paths;
3. SQLite FTS5 matches across title, intent, guidance, and tags.

Scope matches rank first, followed by tags and full-text relevance. This lets a
view-model way recorded under one feature guide a new view model in a distant
directory without loading the full corpus into model context.

## Alignment audit

`rtw check` obtains the changed paths and diff from Git, retrieves at most 12
relevant ways, and sends only that bounded evidence to an isolated judge.

The first pass proposes concrete deviations. A second pass must confirm the
same way, changed path, and line. Invented way IDs, unchanged paths, empty
reasons, malformed JSON, process failures, and unsupported configuration all
fail closed.

## Surfaces

The native binary provides CLI and local stdio MCP. Both call the same Rust
functions. The embedded skill teaches agents when to guide, add, and check but
does not own enforcement.

## Product boundaries

Right This Way does not detect historical failures, define acceptance
criteria, generate architecture, or provide generic code review.

NYA can remember what must not recur. AVP can define what behavior must hold.
RTW independently preserves how this repository already builds a recurring
kind of thing. None requires another.

## Engineering constitution

```text
Production code:         <= 500 LOC
Shared runtime coverage: >= 95%
Packaged entrypoint:     end-to-end smoke tested
Test code:               unlimited
```

`cargo xtask verify` is the canonical local, CI, and release gate.
