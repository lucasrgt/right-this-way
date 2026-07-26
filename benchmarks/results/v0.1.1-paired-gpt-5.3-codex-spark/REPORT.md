# Right This Way Paired Agent Benchmark

Run from `2026-07-26T18:45:30.511707+00:00` to `2026-07-26T18:48:08.118426+00:00` with `codex-cli 0.144.0` on `Linux-6.18.33.1-microsoft-standard-WSL2-x86_64-with-glibc2.36`.

| Case | Baseline | RTW | Guide | Check | Paired improvement |
| --- | --- | --- | --- | --- | --- |
| `design-token` | pass | pass | yes | yes | no |
| `view-model` | pass | pass | yes | yes | no |
| `api-envelope` | pass | pass | yes | yes | no |
| `terraform-tags` | pass | pass | yes | yes | no |
| `http-client` | pass | pass | yes | yes | no |

Baseline pattern deviations: **0**.

RTW pattern deviations: **0**.

Paired improvements: **0 of 0 observed baseline deviations**.

Regressions against passing baselines: **0**.

Overall protocol result: **PASS**.

A paired improvement is counted only when the baseline completes the task with a pattern deviation and the RTW arm completes it in alignment. A baseline pass is a passing tie, never an improvement. Incomplete tasks are reported separately.

The repositories are synthetic, but every task extends a concrete tracked reference implementation and every pattern family links to a primary source. The evaluator remains outside both repositories.

## Pattern sources

| Case | Primary source |
| --- | --- |
| `design-token` | https://designsystem.digital.gov/design-tokens/ |
| `view-model` | https://developer.android.com/topic/architecture/ui-layer/stateholders |
| `api-envelope` | https://www.rfc-editor.org/rfc/rfc9457 |
| `terraform-tags` | https://developer.hashicorp.com/terraform/language/functions/merge |
| `http-client` | https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/ |
