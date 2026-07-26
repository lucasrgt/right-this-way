# Right This Way Large-Corpus Stress Benchmark

Run from `2026-07-26T18:42:18.925319+00:00` to `2026-07-26T18:42:45.647202+00:00` on `Windows-11-10.0.26200-SP0`.

## Corpus and recall

| Metric | Result |
| --- | ---: |
| Versioned ways | 1024 |
| Pattern families | 16 |
| Monorepo surfaces | 64 |
| Positive probes recalled | 128 / 128 |
| Targets ranked first | 128 / 128 |
| Bounded result sets | 128 / 128 |
| Unrelated probes empty | 8 / 8 |
| Cold corrupt-index recovery | 8.156 s |
| Warm recall p50 | 0.094 s |
| Warm recall p95 | 0.125 s |
| Warm recall maximum | 0.235 s |
| Disposable index size | 458752 bytes |

Each positive path has every catalog family in scope. The unique intended pattern must still rank first while the returned context remains bounded.
The first probe starts from a deliberately corrupt index and includes its transactional rebuild. Warm latency excludes that recovery probe.

## Deterministic final check

| Metric | Deviation fixture | Aligned control |
| --- | ---: | ---: |
| Exit code | 1 | 0 |
| Ways presented to judge | 12 | 12 |
| Findings | 1 | 0 |
| Exact target | yes | n/a |
| Latency | 0.375 s | 0.297 s |

The deterministic judge measures RTW orchestration and identity validation. It does not measure semantic model intelligence.

Overall benchmark result: **PASS**.

The generated ways are disclosed scale fixtures. They are not presented as independent production incidents or as a universal adherence rate.
