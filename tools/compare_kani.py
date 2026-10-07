#!/usr/bin/env python3
"""Measure both tools on identical property source, retaining all attempts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
CASES = [("quic", "nexagate-quic"), ("hpack", "nexagate-hpack-after"),
         ("old-hpack", "nexagate-hpack-before"),
         ("assert-forms", "issue-4874-assert-forms"),
         ("false-assert", "issue-4874-false-assert"),
         ("const-generics", "issue-4876-const-generics"),
         ("string-map", "string-map"), ("string-map-bad", "string-map")]


def invoke(command, path, timeout):
    started = time.monotonic()
    process = subprocess.Popen(command, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, start_new_session=True)
    try:
        output, _ = process.communicate(timeout=timeout)
        code = process.returncode
    except subprocess.TimeoutExpired:
        # cargo-kani and Phage launch solver children: terminate the whole
        # attempt, otherwise an inconclusive run keeps consuming resources.
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        output += "\nrunner timeout; inconclusive\n"
        code = None
    path.write_text(output)
    return output, code, time.monotonic() - started


def measure(tool, command, log, timeout):
    output, code, elapsed = invoke(command, log, timeout)
    row = {"tool": tool, "command": command, "exit": code, "wallSeconds": elapsed, "log": str(log)}
    if tool == "kani":
        if code is None:
            status = "unknown-timeout"
        elif "VERIFICATION:- SUCCESSFUL" in output and code == 0:
            status = "proved"
        elif "VERIFICATION:- FAILED" in output:
            status = "counterexample"
        elif "semicolon_in_expressions_from" in output or "trailing semicolon in macro" in output:
            status = "compiler-error-assert-expression"
        else:
            status = "unknown-error"
        solver = re.search(r"Verification Time: ([0-9.]+)s", output)
        row["solverSeconds"] = float(solver[1]) if solver else None
    else:
        status = {0: "proved", 1: "counterexample", 2: "unknown"}.get(code, "unknown-timeout")
        run_line = next((l[5:] for l in output.splitlines() if l.startswith("run: ")), None)
        if run_line and (ROOT / run_line / "RESULT.json").is_file():
            report_path = ROOT / run_line / "RESULT.json"
            report = json.loads(report_path.read_text())
            row["report"] = str(report_path)
            row["solverSeconds"] = report.get("solverWallSeconds")
            log.with_suffix(".json").write_text(json.dumps(report, indent=2) + "\n")
    row["status"] = status
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--cases", nargs="+", default=[case for case, _ in CASES])
    parser.add_argument("--cpu", type=int, help="pin both tools and their children to this CPU")
    parser.add_argument("--baseline", help="retained earlier Phage binary")
    args = parser.parse_args()
    if args.repeats < 1 or any(case not in dict(CASES) for case in args.cases):
        parser.error("positive repeats and known case names are required")
    if args.cpu is not None and args.cpu not in os.sched_getaffinity(0):
        parser.error("selected CPU is not available to this process")
    out = ROOT / "evidence" / f"comparison-{time.time_ns()}"
    out.mkdir()
    source_files = list((ROOT / "src").rglob("*.rs")) + list((ROOT / "comparison").rglob("*.rs"))
    source_files += [Path(__file__), ROOT / "Cargo.toml", ROOT / "comparison/Cargo.toml", ROOT / "target/release/phage"]
    for case in args.cases:
        source_files.append(ROOT / "examples" / (dict(CASES)[case] + ".rs"))
    source_files += list((ROOT / "../Nexagate/crates/gate-http/src/hpack").glob("*.rs"))
    source_files += list((ROOT / "fixtures/nexagate-before/hpack").glob("*.rs"))
    source_files.append(ROOT / "../Nexagate/crates/gate-quic/src/varint.rs")
    if args.baseline:
        source_files.append(ROOT / args.baseline)
    hashes = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files}
    result = {"platform": platform.platform(), "processor": platform.processor(),
              "kani": subprocess.check_output(["cargo", "kani", "--version"], cwd=ROOT, text=True).strip(),
              "rustc": subprocess.check_output(["rustc", "-vV"], text=True).strip(),
              "z3": subprocess.check_output(["z3", "--version"], text=True).strip(),
              "sourceSha256": hashes, "cpuAffinity": args.cpu,
              "hostCpuPressureBefore": Path("/proc/pressure/cpu").read_text().strip(),
              "hostLoadBefore": os.getloadavg(), "repeats": args.repeats, "timeoutSeconds": args.timeout,
              "timingScope": "full CLI process wall time; one unmeasured warmup per tool/case retained",
              "runs": [], "summaries": []}
    record = out / "RESULT.json"
    for case in args.cases:
        source = f"examples/{dict(CASES)[case]}.rs"
        flags = ["--function", "string_map_bad"] if case == "string-map-bad" else []
        commands = [("phage", ["target/release/phage", "check", source, *flags]),
                    ("kani", ["cargo", "kani", "--manifest-path", "comparison/Cargo.toml",
                              "--features", case, "--harness", "shared_property", "--output-format", "terse"])]
        if args.baseline:
            commands.insert(0, ("baseline", [args.baseline, "check", source, *flags]))
        if args.cpu is not None:
            commands = [(tool, ["taskset", "-c", str(args.cpu), *command]) for tool, command in commands]
        for tool, command in commands:
            row = measure(tool, command, out / f"{case}-{tool}-warmup.log", args.timeout)
            row.update(case=case, warmup=True)
            result["runs"].append(row)
            record.write_text(json.dumps(result, indent=2) + "\n")
        # Alternate tool order to reduce consistent scheduling advantage.
        for trial in range(args.repeats):
            ordered = commands if trial % 2 == 0 else list(reversed(commands))
            for tool, command in ordered:
                row = measure(tool, command, out / f"{case}-{tool}-{trial}.log", args.timeout)
                row.update(case=case, trial=trial, warmup=False)
                result["runs"].append(row)
                record.write_text(json.dumps(result, indent=2) + "\n")
        for tool, _ in commands:
            rows = [r for r in result["runs"] if r["case"] == case and r["tool"] == tool and not r["warmup"]]
            summary = {"case": case, "tool": tool, "statuses": [r["status"] for r in rows],
                       "medianWallSeconds": statistics.median(r["wallSeconds"] for r in rows),
                       "minWallSeconds": min(r["wallSeconds"] for r in rows),
                       "maxWallSeconds": max(r["wallSeconds"] for r in rows)}
            result["summaries"].append(summary)
            print(f"{case} {tool}: {summary['statuses']}, median {summary['medianWallSeconds']:.3f}s", flush=True)
        record.write_text(json.dumps(result, indent=2) + "\n")
    result["hostCpuPressureAfter"] = Path("/proc/pressure/cpu").read_text().strip()
    result["hostLoadAfter"] = os.getloadavg()
    record.write_text(json.dumps(result, indent=2) + "\n")
    # Reject a source/binary change during the comparison instead of mixing revisions.
    for path, before in hashes.items():
        if hashlib.sha256(Path(path).read_bytes()).hexdigest() != before:
            raise RuntimeError(f"comparison source changed: {path}")
    print(f"comparison: {out}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
