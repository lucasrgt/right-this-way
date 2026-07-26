# Right This Way Large-Corpus Stress Benchmark

Run from `2026-07-26T18:42:56.412929+00:00` to `2026-07-26T18:45:11.159707+00:00` on `Windows-11-10.0.26200-SP0`.

## Corpus and recall

| Metric | Result |
| --- | ---: |
| Versioned ways | 10000 |
| Pattern families | 16 |
| Monorepo surfaces | 625 |
| Positive probes recalled | 64 / 64 |
| Targets ranked first | 64 / 64 |
| Bounded result sets | 64 / 64 |
| Unrelated probes empty | 8 / 8 |
| Cold corrupt-index recovery | 68.141 s |
| Warm recall p50 | 0.532 s |
| Warm recall p95 | 0.594 s |
| Warm recall maximum | 0.657 s |
| Disposable index size | 4136960 bytes |

Each positive path has every catalog family in scope. The unique intended pattern must still rank first while the returned context remains bounded.
The first probe starts from a deliberately corrupt index and includes its transactional rebuild. Warm latency excludes that recovery probe.

## Deterministic final check

| Metric | Deviation fixture | Aligned control |
| --- | ---: | ---: |
| Exit code | 1 | 0 |
| Ways presented to judge | 12 | 12 |
| Findings | 1 | 0 |
| Exact target | yes | n/a |
| Latency | 0.891 s | 0.828 s |

The deterministic judge measures RTW orchestration and identity validation. It does not measure semantic model intelligence.

Overall benchmark result: **PASS**.

The generated ways are disclosed scale fixtures. They are not presented as independent production incidents or as a universal adherence rate.
