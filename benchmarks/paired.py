#!/usr/bin/env python3
"""Paired coding-agent benchmark for Right This Way."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import random
import re
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


@dataclass(frozen=True)
class Case:
    name: str
    title: str
    intent: str
    guidance: str
    scope: str
    tags: tuple[str, ...]
    references: tuple[str, ...]
    task: str
    source: str
    files: dict[str, str]


CASES = (
    Case(
        "design-token",
        "Action components use semantic tokens",
        "Add a styled action component",
        "Follow PrimaryButton: use role-based color and spacing custom properties, with no literal component colors or spacing.",
        "src/ui/**",
        ("react", "design-token", "component"),
        ("src/ui/PrimaryButton.tsx", "src/ui/tokens.css"),
        "Add src/ui/DangerButton.tsx. It must render a button with label, disabled, and onClick props and match established UI conventions.",
        "https://designsystem.digital.gov/design-tokens/",
        {
            "src/ui/PrimaryButton.tsx": """type Props = {
  label: string;
  disabled?: boolean;
  onClick(): void;
};

export function PrimaryButton({ label, disabled, onClick }: Props) {
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      style={{
        background: "var(--color-action-primary)",
        color: "var(--color-text-on-action)",
        padding: "var(--space-3)",
      }}
    >
      {label}
    </button>
  );
}
""",
            "src/ui/tokens.css": """:root {
  --color-action-primary: #2457d6;
  --color-action-danger: #b42318;
  --color-text-on-action: #ffffff;
  --space-3: 0.75rem;
}
""",
        },
    ),
    Case(
        "view-model",
        "Feature workflows use view models",
        "Add a stateful feature workflow",
        "Follow the order view model: inject the async operation, own transitions, and expose readonly idle, submitting, success, and error state.",
        "src/features/**",
        ("view-model", "state", "typescript"),
        ("src/features/orders/orderViewModel.ts",),
        "Create src/features/payments/paymentViewModel.ts with a createPaymentViewModel factory and an async submit operation.",
        "https://developer.android.com/topic/architecture/ui-layer/stateholders",
        {
            "src/features/orders/orderViewModel.ts": """export type OrderState = Readonly<{
  status: "idle" | "submitting" | "success" | "error";
  message?: string;
}>;

export function createOrderViewModel(saveOrder: () => Promise<void>) {
  let state: OrderState = { status: "idle" };
  return {
    get state(): OrderState {
      return state;
    },
    async submit() {
      state = { status: "submitting" };
      try {
        await saveOrder();
        state = { status: "success" };
      } catch (error) {
        state = { status: "error", message: String(error) };
      }
    },
  };
}
""",
        },
    ),
    Case(
        "api-envelope",
        "API handlers use response helpers",
        "Add an API lookup endpoint",
        "Follow the orders handler: inject the repository and return success and not-found responses through the shared response helpers.",
        "src/api/**",
        ("api", "response-envelope", "python"),
        ("src/api/orders.py", "src/api/responses.py"),
        "Add src/api/invoices.py with get_invoice(repository, invoice_id), returning the repository invoice or a not-found response.",
        "https://www.rfc-editor.org/rfc/rfc9457",
        {
            "src/api/responses.py": """def ok(data):
    return {"data": data, "error": None}


def failure(code, message):
    return {"data": None, "error": {"code": code, "message": message}}
""",
            "src/api/orders.py": """from .responses import failure, ok


def get_order(repository, order_id):
    order = repository.find(order_id)
    if order is None:
        return failure("order_not_found", "Order not found")
    return ok(order)
""",
            "src/api/__init__.py": "",
        },
    ),
    Case(
        "terraform-tags",
        "Terraform resources merge common tags",
        "Add a managed infrastructure resource",
        "Follow the worker module: merge var.common_tags with stable Service and ManagedBy tags on the resource.",
        "infra/modules/**",
        ("terraform", "tags", "infrastructure"),
        ("infra/modules/worker/main.tf", "infra/modules/worker/variables.tf"),
        "Create infra/modules/queue/main.tf and variables.tf for an aws_sqs_queue named from var.name.",
        "https://developer.hashicorp.com/terraform/language/functions/merge",
        {
            "infra/modules/worker/main.tf": """resource "aws_lambda_function" "worker" {
  function_name = var.name
  role          = var.role_arn

  tags = merge(var.common_tags, {
    Service   = "worker"
    ManagedBy = "terraform"
  })
}
""",
            "infra/modules/worker/variables.tf": """variable "name" { type = string }
variable "role_arn" { type = string }
variable "common_tags" { type = map(string) }
""",
        },
    ),
    Case(
        "http-client",
        "HTTP clients share retry and timeout behavior",
        "Add an outbound service client",
        "Follow OrdersClient: call requestWithRetry with an explicit timeout and keep transport behavior out of the feature client.",
        "src/clients/**",
        ("http", "retry", "client"),
        ("src/clients/ordersClient.ts", "src/clients/request.ts"),
        "Add src/clients/invoicesClient.ts exporting getInvoice(baseUrl, invoiceId).",
        "https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/",
        {
            "src/clients/request.ts": """export async function requestWithRetry(
  url: string,
  options: { timeoutMs: number },
) {
  return fetch(url, { signal: AbortSignal.timeout(options.timeoutMs) });
}
""",
            "src/clients/ordersClient.ts": """import { requestWithRetry } from "./request";

export function getOrder(baseUrl: string, orderId: string) {
  return requestWithRetry(`${baseUrl}/orders/${orderId}`, {
    timeoutMs: 5_000,
  });
}
""",
        },
    ),
)


def run(command, cwd: Path, env=None, timeout=120, check=True):
    result = subprocess.run(
        [str(item) for item in command],
        cwd=cwd,
        env=env,
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


def write_repository(root: Path, case: Case):
    root.mkdir(parents=True)
    for relative, body in case.files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8", newline="\n")
    (root / "AGENTS.md").write_text(
        "# Repository instructions\n\n"
        "Make the smallest complete change. Inspect existing conventions and preserve "
        "public behavior unless the task explicitly changes it.\n",
        encoding="utf-8",
        newline="\n",
    )


def seed_rtw(root: Path, case: Case, rtw: Path):
    run([rtw, "init", "--agent-file", "AGENTS.md"], root)
    judge = root.parent / "empty_judge.py"
    judge.write_text(
        'import json\nprint(json.dumps({"deviations": []}))\n',
        encoding="utf-8",
        newline="\n",
    )
    command = json.dumps([sys.executable, str(judge)])
    (root / ".rtw" / "config.local.toml").write_text(
        f"schema = 1\n\n[judge]\ncommand = {command}\n",
        encoding="utf-8",
        newline="\n",
    )
    add = [
        rtw,
        "add",
        "--title",
        case.title,
        "--intent",
        case.intent,
        "--guidance",
        case.guidance,
        "--scope",
        case.scope,
    ]
    for tag in case.tags:
        add += ["--tag", tag]
    for reference in case.references:
        add += ["--reference", reference]
    add += ["--recorded-by", "RTW benchmark"]
    run(add, root)


def initialize(root: Path, case: Case, arm: str, rtw: Path):
    write_repository(root, case)
    run(["git", "init", "-q"], root)
    run(["git", "config", "user.name", "RTW Benchmark"], root)
    run(["git", "config", "user.email", "benchmark@example.test"], root)
    run(["git", "config", "core.autocrlf", "false"], root)
    run(["git", "add", "."], root)
    run(["git", "commit", "-qm", "seed reference implementation"], root)
    if arm == "rtw":
        seed_rtw(root, case, rtw)
        run(["git", "add", "."], root)
        run(["git", "commit", "-qm", "enable Right This Way"], root)
    return run(["git", "rev-parse", "HEAD"], root).stdout.strip()


def command_observed(events: str, subcommand: str):
    pattern = re.compile(rf"\brtw(?:\.exe)?\s+{re.escape(subcommand)}\b", re.I)
    for line in events.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = event.get("item", {})
        if (
            event.get("type") == "item.completed"
            and item.get("type") == "command_execution"
            and item.get("exit_code") == 0
            and item.get("status") == "completed"
            and pattern.search(item.get("command", ""))
        ):
            return True
    return False


def evaluate(case: Case, root: Path):
    if case.name == "design-token":
        path = root / "src/ui/DangerButton.tsx"
        body = path.read_text(encoding="utf-8") if path.is_file() else ""
        task_ok = all(token in body for token in ("DangerButton", "onClick", "disabled"))
        pattern_ok = (
            "var(--color-action-danger)" in body
            and "var(--color-text-on-action)" in body
            and "var(--space-" in body
            and not re.search(r"#[0-9a-f]{3,8}\b|rgba?\(|\b(red|white|black)\b", body, re.I)
        )
        detail = "danger action uses the established semantic color and spacing roles"
    elif case.name == "view-model":
        path = root / "src/features/payments/paymentViewModel.ts"
        body = path.read_text(encoding="utf-8") if path.is_file() else ""
        task_ok = "createPaymentViewModel" in body and "submit" in body
        pattern_ok = (
            "Readonly<" in body
            and all(state in body for state in ('"idle"', '"submitting"', '"success"', '"error"'))
            and re.search(
                r"createPaymentViewModel\s*\([^)]*:\s*\(\)\s*=>\s*Promise\s*<\s*void\s*>",
                body,
            )
            and "fetch(" not in body
        )
        detail = "payment workflow owns readonly transitions and receives its operation"
    elif case.name == "api-envelope":
        path = root / "src/api/invoices.py"
        body = path.read_text(encoding="utf-8") if path.is_file() else ""
        task_ok = "def get_invoice" in body and (".find(" in body or "repository.find" in body)
        pattern_ok = (
            re.search(r"from\s+\.responses\s+import\s+.*\b(ok|failure)\b", body) is not None
            and "ok(" in body
            and "failure(" in body
            and "invoice_not_found" in body
        )
        detail = "invoice handler returns both outcomes through shared response helpers"
    elif case.name == "terraform-tags":
        main = root / "infra/modules/queue/main.tf"
        variables = root / "infra/modules/queue/variables.tf"
        body = main.read_text(encoding="utf-8") if main.is_file() else ""
        vars_body = variables.read_text(encoding="utf-8") if variables.is_file() else ""
        task_ok = (
            'resource "aws_sqs_queue"' in body
            and "var.name" in body
            and re.search(r'variable\s+"name"', vars_body)
        )
        pattern_ok = (
            re.search(r'variable\s+"common_tags"', vars_body)
            and
            "tags = merge(var.common_tags" in re.sub(r"\s+", " ", body)
            and 'Service' in body
            and '"queue"' in body
            and "ManagedBy" in body
            and '"terraform"' in body
        )
        detail = "queue merges common tags with stable service ownership tags"
    else:
        path = root / "src/clients/invoicesClient.ts"
        body = path.read_text(encoding="utf-8") if path.is_file() else ""
        task_ok = "getInvoice" in body and "invoiceId" in body and "/invoices/" in body
        pattern_ok = (
            "requestWithRetry" in body
            and re.search(r"timeoutMs\s*:\s*5[_]?000", body) is not None
            and body.count("fetch(") == 0
            and not re.search(r"\bwhile\s*\(|\bfor\s*\(", body)
        )
        detail = "invoice client delegates bounded transport behavior to the shared helper"
    outcome = "pass" if task_ok and pattern_ok else "pattern_deviation" if task_ok else "incomplete"
    return {
        "outcome": outcome,
        "task_ok": bool(task_ok),
        "pattern_ok": bool(pattern_ok),
        "detail": detail,
    }


def execute(
    case: Case,
    arm: str,
    root: Path,
    output: Path,
    rtw: Path,
    codex: str,
    model,
    codex_home: Path,
):
    baseline = initialize(root, case, arm, rtw)
    trusted = str(root.resolve()).lower()
    if "'" in trusted:
        raise RuntimeError("benchmark path cannot be represented as a trusted project")
    with (codex_home / "config.toml").open("a", encoding="utf-8", newline="\n") as config:
        config.write(
            f"\n[projects.'{trusted}']\n"
            'trust_level = "trusted"\n'
        )
    prompt = (
        "Implement the following task in this repository. Make the smallest complete "
        "change, inspect existing conventions, run relevant checks, and stop when the "
        f"work is ready to commit. Do not ask questions.\n\nTask: {case.task}"
    )
    last = output / f"{case.name}-{arm}-last.md"
    events = output / f"{case.name}-{arm}-events.jsonl"
    env = os.environ.copy()
    env["PATH"] = str(rtw.parent) + os.pathsep + env.get("PATH", "")
    env["CODEX_HOME"] = str(codex_home)
    command = [
        codex,
        "--ask-for-approval",
        "never",
        "exec",
        *([] if model is None else ["--model", model]),
        "--ephemeral",
        "--sandbox",
        "workspace-write",
        "--json",
        "--output-last-message",
        last,
        "-C",
        root,
        prompt,
    ]
    started = time.monotonic()
    attempts = []
    for attempt in range(3):
        result = run(command, root, env=env, timeout=420, check=False)
        attempts.append(result)
        if result.returncode == 0 or "at capacity" not in (result.stdout + result.stderr).lower():
            break
        time.sleep(5 * (attempt + 1))
    seconds = round(time.monotonic() - started, 3)
    events.write_text(
        "".join(value.stdout.rstrip() + "\n" for value in attempts),
        encoding="utf-8",
        newline="\n",
    )
    (output / f"{case.name}-{arm}-stderr.log").write_text(
        "".join(value.stderr.rstrip() + "\n" for value in attempts if value.stderr),
        encoding="utf-8",
        newline="\n",
    )
    untracked = run(
        ["git", "ls-files", "--others", "--exclude-standard"], root
    ).stdout.splitlines()
    if untracked:
        run(["git", "add", "-N", "--", *untracked], root)
    (output / f"{case.name}-{arm}.diff").write_text(
        run(["git", "diff", "--binary", baseline, "--"], root).stdout,
        encoding="utf-8",
        newline="\n",
    )
    evaluation = evaluate(case, root)
    evaluation.update(
        {
            "case": case.name,
            "arm": arm,
            "agent_exit": result.returncode,
            "seconds": seconds,
            "guide_observed": command_observed(result.stdout, "guide"),
            "check_observed": command_observed(result.stdout, "check"),
        }
    )
    return evaluation


def render(summary):
    by_case = {}
    for item in summary["results"]:
        by_case.setdefault(item["case"], {})[item["arm"]] = item
    lines = [
        "# Right This Way Paired Agent Benchmark",
        "",
        f"Run from `{summary['started_at']}` to `{summary['completed_at']}` with "
        f"`{summary['agent']}` on `{summary['platform']}`.",
        "",
        "| Case | Baseline | RTW | Guide | Check | Paired improvement |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    case_names = {item["case"] for item in summary["results"]}
    for case in (case for case in CASES if case.name in case_names):
        baseline = by_case[case.name]["baseline"]
        guided = by_case[case.name]["rtw"]
        improved = baseline["outcome"] == "pattern_deviation" and guided["outcome"] == "pass"
        lines.append(
            f"| `{case.name}` | {baseline['outcome']} | {guided['outcome']} | "
            f"{'yes' if guided['guide_observed'] else 'no'} | "
            f"{'yes' if guided['check_observed'] else 'no'} | "
            f"{'yes' if improved else 'no'} |"
        )
    lines += [
        "",
        f"Baseline pattern deviations: **{summary['baseline_deviations']}**.",
        "",
        f"RTW pattern deviations: **{summary['rtw_deviations']}**.",
        "",
        f"Paired improvements: **{summary['paired_improvements']} of "
        f"{summary['baseline_deviations']} observed baseline deviations**.",
        "",
        f"Regressions against passing baselines: **{summary['regressions']}**.",
        "",
        f"Overall protocol result: **{'PASS' if summary['passed'] else 'FAIL'}**.",
        "",
        "A paired improvement is counted only when the baseline completes the task with "
        "a pattern deviation and the RTW arm completes it in alignment. A baseline pass "
        "is a passing tie, never an improvement. Incomplete tasks are reported separately.",
        "",
        "The repositories are synthetic, but every task extends a concrete tracked "
        "reference implementation and every pattern family links to a primary source. "
        "The evaluator remains outside both repositories.",
        "",
        "## Pattern sources",
        "",
        "| Case | Primary source |",
        "| --- | --- |",
    ]
    lines += [f"| `{case.name}` | {case.source} |" for case in CASES]
    lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rtw", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--codex", default=shutil.which("codex") or "codex")
    parser.add_argument("--model")
    parser.add_argument("--case", action="append", choices=[case.name for case in CASES])
    parser.add_argument("--seed", type=int, default=20260726)
    parser.add_argument("--work-parent", type=Path, default=Path.cwd().parent)
    parser.add_argument("--keep-worktree", action="store_true")
    args = parser.parse_args()
    rtw = args.rtw.resolve()
    output = args.output.resolve()
    if not rtw.is_file():
        raise SystemExit(f"rtw binary not found: {rtw}")
    if output.exists() and any(output.iterdir()):
        raise SystemExit(f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)
    selected = [case for case in CASES if not args.case or case.name in args.case]
    order = [(case, arm) for case in selected for arm in ("baseline", "rtw")]
    random.Random(args.seed).shuffle(order)
    started_at = datetime.now(timezone.utc).isoformat()
    work_parent = args.work_parent.resolve()
    work_parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="rtw-paired-", dir=work_parent))
    results = []
    try:
        codex_home = work / "codex-home"
        codex_home.mkdir()
        configured_home = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
        auth = configured_home / "auth.json"
        if auth.is_file():
            shutil.copy2(auth, codex_home / "auth.json")
        elif "OPENAI_API_KEY" not in os.environ:
            raise SystemExit("Codex authentication not found")
        codex_config = (
            'approval_policy = "never"\n'
            'sandbox_mode = "workspace-write"\n'
        )
        if os.name == "nt":
            codex_config += '\n[windows]\nsandbox = "unelevated"\n'
        (codex_home / "config.toml").write_text(
            codex_config,
            encoding="utf-8",
            newline="\n",
        )
        for index, (case, arm) in enumerate(order, 1):
            print(f"[{index}/{len(order)}] {case.name} {arm}", flush=True)
            results.append(
                execute(
                    case,
                    arm,
                    work / f"{case.name}-{arm}",
                    output,
                    rtw,
                    args.codex,
                    args.model,
                    codex_home,
                )
            )
        by_case = {}
        for item in results:
            by_case.setdefault(item["case"], {})[item["arm"]] = item
        baseline_deviations = sum(
            pair["baseline"]["outcome"] == "pattern_deviation" for pair in by_case.values()
        )
        rtw_deviations = sum(
            pair["rtw"]["outcome"] == "pattern_deviation" for pair in by_case.values()
        )
        paired_improvements = sum(
            pair["baseline"]["outcome"] == "pattern_deviation"
            and pair["rtw"]["outcome"] == "pass"
            for pair in by_case.values()
        )
        regressions = sum(
            pair["baseline"]["outcome"] == "pass"
            and pair["rtw"]["outcome"] != "pass"
            for pair in by_case.values()
        )
        passed = (
            all(pair["rtw"]["outcome"] == "pass" for pair in by_case.values())
            and all(pair["rtw"]["guide_observed"] for pair in by_case.values())
            and all(pair["rtw"]["check_observed"] for pair in by_case.values())
            and regressions == 0
        )
        summary = {
            "schema": 1,
            "benchmark": "paired-agent-pattern-adherence",
            "started_at": started_at,
            "completed_at": datetime.now(timezone.utc).isoformat(),
            "agent": run([args.codex, "--version"], Path.cwd()).stdout.strip(),
            "model": args.model or "Codex CLI default",
            "codex_home": "isolated authentication-only home",
            "rtw": {
                "version": run([rtw, "--version"], Path.cwd()).stdout.strip(),
                "sha256": hashlib.sha256(rtw.read_bytes()).hexdigest(),
            },
            "platform": platform.platform(),
            "seed": args.seed,
            "order": [f"{case.name}:{arm}" for case, arm in order],
            "baseline_deviations": baseline_deviations,
            "rtw_deviations": rtw_deviations,
            "paired_improvements": paired_improvements,
            "regressions": regressions,
            "results": results,
            "passed": passed,
        }
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
