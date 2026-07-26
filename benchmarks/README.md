# Right This Way Benchmarks

These benchmarks test two separate claims:

1. a coding agent can use a versioned way to preserve an existing repository
   pattern;
2. RTW can retrieve and audit bounded evidence from a large corpus without
   silently losing the relevant way.

They do not claim that every model follows every pattern or that generated ways
represent thousands of independent production decisions.

## Suite

| Benchmark | What it measures | Judge |
| --- | --- | --- |
| Paired agent benchmark | Pattern adherence with and without RTW | External deterministic evaluator |
| Large-corpus stress benchmark | Retrieval, ranking, bounded output, index recovery, and audit contracts | Deterministic protocol judge |

Every committed result directory contains a human-readable report,
machine-readable summary, inputs or diffs needed for inspection, and candidate
version and SHA-256 metadata.

## Published v0.1.1 results

| Run | Result | Evidence |
| --- | --- | --- |
| [Paired agent](results/v0.1.1-paired-gpt-5.3-codex-spark/REPORT.md) | PASS | 5/5 RTW arms passed, 5/5 guides and checks observed, zero regressions |
| [1,024-way stress](results/v0.1.1-stress-1024/REPORT.md) | PASS | 128/128 targets ranked first, 8/8 unrelated probes empty |
| [10,000-way stress](results/v0.1.1-stress-10000/REPORT.md) | PASS | 64/64 targets ranked first, 8/8 unrelated probes empty |

All five paired baselines also passed. They are reported as passing ties, so
the run counts zero paired improvements rather than attributing already-correct
work to RTW. The 10,000-way run recovered a deliberately corrupt 4.14 MB index
in 68.141 seconds; warm retrieval measured 0.532 seconds p50, 0.594 seconds p95,
and 0.657 seconds maximum on the recorded host. These timings describe that
run, not a service-level guarantee.

## Paired agent benchmark

The paired protocol creates two repositories from the same seed for each case:

| Arm | Additional state |
| --- | --- |
| Baseline | Reference implementation and ordinary repository instructions |
| RTW | The same seed plus one versioned way, the managed skill, and managed instructions |

Both arms receive the same task, model, prompt, and execution limits. Their
order is randomized from a recorded seed. The Codex CLI runs inside a fresh
Docker container with an isolated authentication-only home, workspace-write
sandboxing, no host source checkout, and no Docker socket.

The RTW arm must successfully execute `rtw guide`. It also executes `rtw check`
through the repository workflow. A deterministic empty check judge prevents a
second model call from contaminating the adherence comparison. The external
evaluator independently inspects the resulting repository and does not receive
the arm label.

### Cases

| Case | Pattern under test | Domain |
| --- | --- | --- |
| `design-token` | Semantic color and spacing tokens | React UI |
| `view-model` | Injected operation and readonly workflow state | TypeScript architecture |
| `api-envelope` | Shared success and failure response helpers | Python backend |
| `terraform-tags` | Common and stable ownership tags | Infrastructure |
| `http-client` | Shared bounded retry and timeout transport | TypeScript client |

Each case starts from a concrete tracked reference implementation. The task
asks for analogous work without spelling out the hidden pattern.

### Outcome taxonomy

| Outcome | Meaning |
| --- | --- |
| `pass` | The requested task and repository pattern are both present |
| `pattern_deviation` | The task is complete but the established pattern is not preserved |
| `incomplete` | The requested task itself is incomplete |

A paired improvement is counted only when the baseline is
`pattern_deviation` and the corresponding RTW arm is `pass`. A baseline pass is
a passing tie and is never reported as an RTW prevention. A passing protocol
also requires every RTW arm to pass, every RTW `guide` and `check` invocation
to be observed, and zero regressions against passing baselines.

### Reproduce

Build a Linux RTW binary, then:

```bash
docker build \
  --tag rtw-benchmark:local \
  --file benchmarks/Dockerfile \
  benchmarks

mkdir -p benchmarks/results/local-paired

docker run --rm \
  --security-opt seccomp=unconfined \
  --mount type=bind,src="$HOME/.codex/auth.json",dst=/seed/auth.json,readonly \
  --mount type=bind,src="$(pwd)/benchmarks/paired.py",dst=/benchmarks/paired.py,readonly \
  --mount type=bind,src="$(pwd)/target/release/rtw",dst=/usr/local/bin/rtw,readonly \
  --mount type=bind,src="$(pwd)/benchmarks/results/local-paired",dst=/output \
  --workdir /work \
  rtw-benchmark:local \
  python3 /benchmarks/paired.py \
  --rtw /usr/local/bin/rtw \
  --output /output \
  --model gpt-5.3-codex-spark \
  --work-parent /work
```

The model name is an explicit input, not a required RTW dependency. A current
Codex authentication file and model access are required to reproduce this
specific agent run.

## Large-corpus stress benchmark

The stress protocol expands 16 documented pattern families across synthetic
monorepo surfaces. Every positive probe has all 16 families in scope. The
unique intended family must still be recalled, rank first, and fit within the
eight-way context limit.

| Dimension | Coverage |
| --- | --- |
| Frontend | useMemo, design tokens, localized messages |
| Architecture | Feature view-model state |
| Backend | API envelopes, parameterized queries, tenant cache keys |
| Clients | Retry and timeout policy |
| Operations | Structured logging |
| Infrastructure | Terraform tags and Kubernetes probes |
| Data and science | Validated boundaries and explicit units |
| Concurrency | Cooperative cancellation |
| Testing | Shared fixture builders |
| Documentation | Executable command examples |

Eight unrelated probes must return no candidates. Before the first positive
probe, the benchmark deliberately corrupts `.rtw/index.sqlite`; the reported
cold time therefore includes recovery and a transactional rebuild. Warm p50,
p95, and maximum latency exclude that first recovery probe.

The final audit uses one explicit deviation and one aligned control. The judge
may return only the selected way and changed path. This validates RTW
orchestration and identity checks, not semantic model intelligence.

### Defect discovered by the protocol

The first 10,000-way run failed because one of 64 targets ranked second. RTW
was limiting the global FTS result set before applying scope and tag ranking,
so more than 64 semantically identical ways could exclude the correct scoped
candidate. The corrected retrieval uses the complete internal FTS match set
and deterministic local term overlap before bounding agent context. A
permanent test now reproduces more than 64 full-text ties.

### Reproduce 1,024 ways

```bash
python3 benchmarks/stress.py \
  --rtw target/release/rtw \
  --contexts 64 \
  --probes 128 \
  --output benchmarks/results/local-stress-1024
```

### Reproduce 10,000 ways

```bash
python3 benchmarks/stress.py \
  --rtw target/release/rtw \
  --contexts 625 \
  --probes 64 \
  --output benchmarks/results/local-stress-10000
```

The catalog and every primary source are declared in
[`catalog.json`](catalog.json).

## Result artifacts

| Artifact | Purpose |
| --- | --- |
| `REPORT.md` | Human-readable outcome and limitations |
| `summary.json` | Candidate identity, protocol inputs, metrics, and pass state |
| `recall-probes.jsonl` | Target identity, rank, result count, and latency per probe |
| `*.diff` | Exact agent or stress-fixture changes evaluated |
| `*-events.jsonl` | Agent command and file-change event stream |
| `*-stderr.log` | Non-JSON agent process diagnostics |
| `*-last.md` | Agent's final response |
| `*-check.json` | Deterministic RTW audit output |

Generated stress ways are disclosed scale fixtures. Paired repositories are
synthetic but use concrete references and documented engineering patterns.
Results are auditable evidence for these protocols, not a universal prevention
or adherence rate.
