#!/usr/bin/env python3
"""Deterministic large-corpus benchmark for Right This Way."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path


SURFACE_GROUPS = ("apps", "services", "packages", "infra", "data", "science", "clients", "docs")


def run(command, cwd: Path, timeout=180, check=True):
    result = subprocess.run(
        [str(item) for item in command],
        cwd=cwd,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        timeout=timeout,
    )
    if check and result.returncode:
        raise RuntimeError(
            f"{' '.join(map(str, command))} failed with {result.returncode}\n"
            f"{result.stdout}\n{result.stderr}"
        )
    return result


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, math.ceil(len(ordered) * fraction) - 1)]


def quote(value):
    return json.dumps(value, ensure_ascii=False)


def surface(index):
    return f"{SURFACE_GROUPS[index % len(SURFACE_GROUPS)]}/surface-{index:04d}"


def way_id(context_index, family_index):
    return f"rtw-stress-{context_index:04d}-{family_index:02d}"


def way_text(item, context_index, family_index, commit):
    context = surface(context_index)
    tags = item["tags"] + [f"pattern{family_index:02d}", f"surface{context_index:04d}"]
    lines = [
        "schema = 1",
        f"id = {quote(way_id(context_index, family_index))}",
        f"title = {quote(item['title'])}",
        f"intent = {quote(item['intent'])}",
        f"guidance = {quote(item['guidance'])}",
        f"scopes = [{quote(context + '/**')}]",
        f"tags = [{', '.join(quote(tag) for tag in tags)}]",
        f"references = [{quote(context + '/reference.txt')}]",
        'recorded_at = "2026-01-01T00:00:00Z"',
        'recorded_by = "RTW benchmark"',
        f"recorded_commit = {quote(commit)}",
        "",
    ]
    return "\n".join(lines)


def initialize(root: Path, rtw: Path, catalog, contexts):
    root.mkdir(parents=True)
    run(["git", "init", "-q"], root)
    run(["git", "config", "user.name", "RTW Benchmark"], root)
    run(["git", "config", "user.email", "benchmark@example.test"], root)
    run(["git", "config", "core.autocrlf", "false"], root)
    (root / "AGENTS.md").write_text(
        "# Benchmark repository\n\n"
        "Make the smallest complete change and preserve established repository patterns.\n",
        encoding="utf-8",
        newline="\n",
    )
    for context_index in range(contexts):
        path = root / surface(context_index) / "reference.txt"
        path.parent.mkdir(parents=True)
        body = "\n".join(
            f"{item['key']}: {item['guidance']}" for item in catalog
        )
        path.write_text(body + "\n", encoding="utf-8", newline="\n")
    run(["git", "add", "."], root)
    run(["git", "commit", "-qm", "seed proven reference implementations"], root)
    commit = run(["git", "rev-parse", "HEAD"], root).stdout.strip()
    run([rtw, "init", "--agent-file", "AGENTS.md"], root)
    ways = root / ".rtw" / "ways"
    for context_index in range(contexts):
        for family_index, item in enumerate(catalog):
            identifier = way_id(context_index, family_index)
            (ways / f"{identifier}.toml").write_text(
                way_text(item, context_index, family_index, commit),
                encoding="utf-8",
                newline="\n",
            )
    run(["git", "add", "."], root)
    run(["git", "commit", "-qm", f"seed {contexts * len(catalog)} ways"], root)


def guide(rtw: Path, root: Path, task: str, path: str, limit=8):
    started = time.monotonic()
    result = run(
        [rtw, "guide", "--task", task, "--path", path, "--limit", str(limit), "--json"],
        root,
        timeout=300,
    )
    elapsed = time.monotonic() - started
    return json.loads(result.stdout), elapsed


def configure_judge(root: Path, target_id: str, target_path: str):
    judge = root.parent / "deterministic_judge.py"
    judge.write_text(
        """import json,sys
target_id,target_path=sys.argv[1:3]
prompt=sys.stdin.read()
deviations=[]
if "DEVIATES_FROM_WAY" in prompt:
    deviations=[{"way_id":target_id,"path":target_path,"line":1,
                 "reason":"fixture explicitly deviates from the selected proven pattern"}]
print(json.dumps({"deviations":deviations}))
""",
        encoding="utf-8",
        newline="\n",
    )
    command = json.dumps([sys.executable, str(judge), target_id, target_path])
    (root / ".rtw" / "config.local.toml").write_text(
        f"schema = 1\n\n[judge]\ncommand = {command}\n",
        encoding="utf-8",
        newline="\n",
    )


def execute_check(rtw: Path, root: Path, output: Path, label: str, task: str):
    started = time.monotonic()
    result = run(
        [rtw, "check", "--task", task, "--json"],
        root,
        timeout=300,
        check=False,
    )
    elapsed = round(time.monotonic() - started, 3)
    (output / f"{label}-check.json").write_text(
        result.stdout, encoding="utf-8", newline="\n"
    )
    (output / f"{label}-check.stderr.log").write_text(
        result.stderr, encoding="utf-8", newline="\n"
    )
    try:
        verdict = json.loads(result.stdout)
    except json.JSONDecodeError:
        verdict = {"ways_checked": None, "deviations": []}
    return result.returncode, elapsed, verdict


def render(summary):
    recall = summary["recall"]
    positive = summary["check"]["positive"]
    negative = summary["check"]["negative"]
    lines = [
        "# Right This Way Large-Corpus Stress Benchmark",
        "",
        f"Run from `{summary['started_at']}` to `{summary['completed_at']}` on "
        f"`{summary['platform']}`.",
        "",
        "## Corpus and recall",
        "",
        "| Metric | Result |",
        "| --- | ---: |",
        f"| Versioned ways | {summary['corpus_size']} |",
        f"| Pattern families | {summary['families']} |",
        f"| Monorepo surfaces | {summary['contexts']} |",
        f"| Positive probes recalled | {recall['targets_recalled']} / {recall['probes']} |",
        f"| Targets ranked first | {recall['ranked_first']} / {recall['probes']} |",
        f"| Bounded result sets | {recall['bounded']} / {recall['probes']} |",
        f"| Unrelated probes empty | {recall['negative_empty']} / {recall['negative_probes']} |",
        f"| Cold corrupt-index recovery | {recall['cold_rebuild_seconds']} s |",
        f"| Warm recall p50 | {recall['warm_latency_seconds']['p50']} s |",
        f"| Warm recall p95 | {recall['warm_latency_seconds']['p95']} s |",
        f"| Warm recall maximum | {recall['warm_latency_seconds']['max']} s |",
        f"| Disposable index size | {recall['index_bytes']} bytes |",
        "",
        "Each positive path has every catalog family in scope. The unique intended "
        "pattern must still rank first while the returned context remains bounded.",
        "The first probe starts from a deliberately corrupt index and includes its "
        "transactional rebuild. Warm latency excludes that recovery probe.",
        "",
        "## Deterministic final check",
        "",
        "| Metric | Deviation fixture | Aligned control |",
        "| --- | ---: | ---: |",
        f"| Exit code | {positive['exit']} | {negative['exit']} |",
        f"| Ways presented to judge | {positive['ways_checked']} | {negative['ways_checked']} |",
        f"| Findings | {positive['findings']} | {negative['findings']} |",
        f"| Exact target | {'yes' if positive['exact_target'] else 'no'} | n/a |",
        f"| Latency | {positive['seconds']} s | {negative['seconds']} s |",
        "",
        "The deterministic judge measures RTW orchestration and identity validation. "
        "It does not measure semantic model intelligence.",
        "",
        f"Overall benchmark result: **{'PASS' if summary['passed'] else 'FAIL'}**.",
        "",
        "The generated ways are disclosed scale fixtures. They are not presented as "
        "independent production incidents or as a universal adherence rate.",
        "",
    ]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rtw", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--catalog", type=Path, default=Path(__file__).with_name("catalog.json")
    )
    parser.add_argument("--contexts", type=int, default=64)
    parser.add_argument("--probes", type=int, default=128)
    parser.add_argument("--keep-worktree", action="store_true")
    args = parser.parse_args()
    rtw = args.rtw.resolve()
    output = args.output.resolve()
    if not rtw.is_file():
        raise SystemExit(f"rtw binary not found: {rtw}")
    if args.contexts < 1 or args.probes < 1:
        raise SystemExit("contexts and probes must be positive")
    if output.exists() and any(output.iterdir()):
        raise SystemExit(f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)
    catalog = json.loads(args.catalog.read_text(encoding="utf-8"))
    if len(catalog) != 16:
        raise SystemExit("the benchmark catalog must contain exactly 16 families")
    total = len(catalog) * args.contexts
    probes = min(args.probes, total)
    started_at = datetime.now(timezone.utc).isoformat()
    work = Path(tempfile.mkdtemp(prefix="rtw-stress-"))
    root = work / "repository"
    try:
        initialize(root, rtw, catalog, args.contexts)
        (root / ".rtw" / "index.sqlite").write_bytes(b"deliberately corrupt")
        selected = sorted({(offset * total) // probes for offset in range(probes)})
        probe_results = []
        latencies = []
        for offset, flat_index in enumerate(selected, 1):
            context_index, family_index = divmod(flat_index, len(catalog))
            item = catalog[family_index]
            identifier = way_id(context_index, family_index)
            path = f"{surface(context_index)}/new/{item['key']}.txt"
            recalled, seconds = guide(
                rtw,
                root,
                f"Apply pattern{family_index:02d} for {item['key']}",
                path,
            )
            ids = [way["id"] for way in recalled]
            latencies.append(seconds)
            probe_results.append(
                {
                    "target": identifier,
                    "path": path,
                    "rank": ids.index(identifier) if identifier in ids else None,
                    "candidate_count": len(ids),
                    "seconds": round(seconds, 4),
                }
            )
            print(f"[recall {offset}/{len(selected)}] {identifier}", flush=True)
        negative_results = []
        for index, group in enumerate(SURFACE_GROUPS):
            recalled, seconds = guide(
                rtw,
                root,
                "zqxv nebula quasar",
                f"unrelated/void-{index}/unknown.bin",
            )
            negative_results.append(
                {"group": group, "candidate_count": len(recalled), "seconds": round(seconds, 4)}
            )
            latencies.append(seconds)
        target_context = args.contexts - 1
        target_family = len(catalog) - 1
        target_item = catalog[target_family]
        target_id = way_id(target_context, target_family)
        target_path = f"{surface(target_context)}/new/{target_item['key']}.txt"
        configure_judge(root, target_id, target_path)
        changed = root / target_path
        changed.parent.mkdir(parents=True, exist_ok=True)
        changed.write_text(
            f"DEVIATES_FROM_WAY {target_id}\n", encoding="utf-8", newline="\n"
        )
        run(["git", "add", "-N", "--", target_path], root)
        (output / "positive.diff").write_text(
            run(["git", "diff", "--no-ext-diff", "HEAD", "--"], root).stdout,
            encoding="utf-8",
            newline="\n",
        )
        task = f"Apply pattern{target_family:02d} for {target_item['key']}"
        positive_exit, positive_seconds, positive = execute_check(
            rtw, root, output, "positive", task
        )
        changed.write_text(
            f"ALIGNED_WITH_WAY {target_id}\n", encoding="utf-8", newline="\n"
        )
        (output / "negative.diff").write_text(
            run(["git", "diff", "--no-ext-diff", "HEAD", "--"], root).stdout,
            encoding="utf-8",
            newline="\n",
        )
        negative_exit, negative_seconds, negative = execute_check(
            rtw, root, output, "negative", task
        )
        deviations = positive.get("deviations", [])
        exact_target = any(
            item.get("way_id") == target_id and item.get("path") == target_path
            for item in deviations
        )
        warm_latencies = latencies[1:] or latencies
        recall_result = {
            "probes": len(probe_results),
            "targets_recalled": sum(item["rank"] is not None for item in probe_results),
            "ranked_first": sum(item["rank"] == 0 for item in probe_results),
            "bounded": sum(item["candidate_count"] <= 8 for item in probe_results),
            "negative_probes": len(negative_results),
            "negative_empty": sum(item["candidate_count"] == 0 for item in negative_results),
            "cold_rebuild_seconds": round(latencies[0], 4),
            "warm_latency_seconds": {
                "p50": round(statistics.median(warm_latencies), 4),
                "p95": round(percentile(warm_latencies, 0.95), 4),
                "max": round(max(warm_latencies), 4),
            },
            "index_bytes": (root / ".rtw" / "index.sqlite").stat().st_size,
        }
        check_result = {
            "positive": {
                "exit": positive_exit,
                "seconds": positive_seconds,
                "ways_checked": positive.get("ways_checked"),
                "findings": len(deviations),
                "exact_target": exact_target,
            },
            "negative": {
                "exit": negative_exit,
                "seconds": negative_seconds,
                "ways_checked": negative.get("ways_checked"),
                "findings": len(negative.get("deviations", [])),
            },
        }
        passed = (
            recall_result["targets_recalled"] == recall_result["probes"]
            and recall_result["ranked_first"] == recall_result["probes"]
            and recall_result["bounded"] == recall_result["probes"]
            and recall_result["negative_empty"] == recall_result["negative_probes"]
            and positive_exit == 1
            and exact_target
            and len(deviations) == 1
            and negative_exit == 0
            and not negative.get("deviations")
        )
        completed_at = datetime.now(timezone.utc).isoformat()
        summary = {
            "schema": 1,
            "benchmark": "large-corpus-stress",
            "started_at": started_at,
            "completed_at": completed_at,
            "platform": platform.platform(),
            "candidate": {
                "version": run([rtw, "--version"], root).stdout.strip(),
                "sha256": hashlib.sha256(rtw.read_bytes()).hexdigest(),
            },
            "corpus_size": total,
            "families": len(catalog),
            "contexts": args.contexts,
            "recall": recall_result,
            "check": check_result,
            "passed": passed,
        }
        (output / "recall-probes.jsonl").write_text(
            "".join(json.dumps(item) + "\n" for item in probe_results),
            encoding="utf-8",
            newline="\n",
        )
        (output / "negative-probes.json").write_text(
            json.dumps(negative_results, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        (output / "summary.json").write_text(
            json.dumps(summary, indent=2) + "\n", encoding="utf-8", newline="\n"
        )
        report = render(summary)
        (output / "REPORT.md").write_text(report, encoding="utf-8", newline="\n")
        print(report)
        if not passed:
            raise SystemExit(1)
    finally:
        if args.keep_worktree:
            print(f"worktree={work}", flush=True)
        else:
            shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
