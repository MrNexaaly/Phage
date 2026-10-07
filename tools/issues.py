#!/usr/bin/env python3
"""Protect behavior learned from upstream issues without claiming all fixed."""
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
CASES = [
    ("4874-positive", "examples/issue-4874-assert-forms.rs", [], 0, None),
    ("4874-negative", "examples/issue-4874-false-assert.rs", [], 1, "rustPanic"),
    ("4876-explicit", "examples/issue-4876-const-generics.rs", [], 0, None),
    ("1423-missing-function", "examples/unsupported.rs", [], 2, "unsupportedCall"),
    ("4867-vector-unknown", "fixtures/issue-4867-pointer-vector.ll", [], 2, "unsupportedType"),
    ("4877-block-limit", "fixtures/reachable-loop.ll", ["--blockVisits", "2"], 2, "blockLimit"),
    ("4877-state-limit", "fixtures/reachable-loop.ll", ["--maxStates", "1"], 2, "stateLimit"),
]


def main():
    out = ROOT / "evidence" / f"issues-{time.time_ns()}"
    out.mkdir()
    results = []
    for case, source, flags, expected, reason in CASES:
        command = ["target/release/phage", "check", source, *flags]
        process = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=60)
        output = process.stdout + process.stderr
        (out / f"{case}.log").write_text(output)
        run = next((line[5:] for line in output.splitlines() if line.startswith("run: ")), None)
        report = json.loads((ROOT / run / "RESULT.json").read_text()) if run else {}
        (out / f"{case}.json").write_text(json.dumps(report, indent=2) + "\n")
        codes = [d["code"] for d in report.get("diagnostics", [])]
        passed = process.returncode == expected and (reason is None or reason in codes)
        if case == "1423-missing-function":
            passed &= "external_result" in report.get("detail", "")
        if case == "4874-negative":
            # The exact assertion source line must survive every report layer.
            passed &= report["diagnostics"][0]["source"]["line"] == 8
        row = {"case": case, "command": command, "exit": process.returncode,
               "expectedExit": expected, "reasonCodes": codes, "passed": passed}
        results.append(row)
        (out / "RESULT.json").write_text(json.dumps(results, indent=2) + "\n")
        print(f"{case}: {'passed' if passed else 'FAILED'}", flush=True)
    print(f"issue controls: {out}", flush=True)
    return int(not all(row["passed"] for row in results))


if __name__ == "__main__":
    raise SystemExit(main())
