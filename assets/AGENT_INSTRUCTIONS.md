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
