#!/usr/bin/env python3
"""Run examples and require retained reports, hashes and counterexample replays.

--standalone excludes only the two cases that import the sibling Nexagate tree.
Every attempt, including validation failures and timeouts, gets a unique log.
"""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
CASES = [("safe-add", 0), ("overflow", 1), ("nexagate-quic", 0),
         ("nexagate-hpack-before", 1), ("nexagate-hpack-after", 0), ("unsupported", 2),
         ("issue-4874-assert-forms", 0), ("issue-4874-false-assert", 1),
         ("issue-4876-const-generics", 0), ("intrinsics", 0), ("string-map", 0)]
SIBLING_CASES = {"nexagate-quic", "nexagate-hpack-after"}
STATUSES = {0: "proved", 1: "counterexample", 2: "unknown"}
CONFIRMATIONS = {"confirmed: property returned false", "confirmed: Rust panicked"}


def selected_cases(standalone):
    return [(name, code) for name, code in CASES
            if not standalone or name not in SIBLING_CASES]


def run(command, logfile, timeout=180, env=None):
    started = time.monotonic()
    try:
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=timeout, env=env)
        output = result.stdout + result.stderr
        code = result.returncode
    except subprocess.TimeoutExpired as error:
        def text(value):
            return value.decode(errors="replace") if isinstance(value, bytes) else (value or "")
        output = text(error.stdout) + text(error.stderr) + "\nrunner timeout; inconclusive\n"
        code = None
    except OSError as error:
        output = f"runner could not execute command: {error}\n"
        code = None
    logfile.write_text(output)
    return code, output, time.monotonic() - started


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_report(output, name, expected, code, binary):
    if code != expected:
        raise ValueError(f"exit {code}, expected {expected}")
    # Use the actual report announcement, not an inferred path from 'run:'.
    reports = [line[8:].strip() for line in output.splitlines() if line.startswith("report: ")]
    if len(reports) != 1 or not reports[0]:
        raise ValueError("expected exactly one report: announcement")
    path = (ROOT / reports[0]).resolve()
    report = json.loads(path.read_text())
    if not isinstance(report, dict) or report.get("schema") != 4:
        raise ValueError("expected report schema 4")
    if report.get("status") != STATUSES[expected] or report.get("engine") != "llvm":
        raise ValueError("report status/engine does not match expected result")
    source = (ROOT / "examples" / f"{name}.rs").resolve()
    if report.get("source") != str(source) or report.get("function") != "phage_target":
        raise ValueError("report source/function does not match case")
    fingerprints = report.get("fingerprints")
    if not isinstance(fingerprints, list) or not fingerprints:
        raise ValueError("missing source fingerprints")
    paths = set()
    for item in fingerprints:
        if not isinstance(item, dict) or not isinstance(item.get("path"), str):
            raise ValueError("invalid fingerprint")
        original = Path(item["path"])
        if not original.is_absolute() or original in paths:
            raise ValueError("fingerprint paths must be absolute and unique")
        paths.add(original)
        if digest(original) != item.get("sha256"):
            raise ValueError(f"source changed: {original}")
        if original.suffix == ".rs":
            snapshot = path.parent / "sources" / original.relative_to("/")
            if digest(snapshot) != item["sha256"]:
                raise ValueError(f"snapshot changed: {snapshot}")
    compiler = Path(report.get("compilerPath", ""))
    if not compiler.is_absolute() or compiler not in paths:
        raise ValueError("report must fingerprint its absolute resolved compiler")
    if source not in paths or binary.resolve() not in paths:
        raise ValueError("report must fingerprint the case and verifier binary")
    artifact_hash = digest(path.parent / "artifact.ll")
    if artifact_hash != report.get("artifactSha256") or artifact_hash != report.get("llvmSha256"):
        raise ValueError("LLVM artifact hash mismatch")
    return path, report


def check_case(name, expected, binary, directory, toolchain=None, compiler=None, compiler_path=None):
    def execute(command, logfile):
        if toolchain is None:
            return run(command, logfile)
        return run(command, logfile, env=dict(os.environ, RUSTUP_TOOLCHAIN=toolchain))

    code, output, elapsed = execute([str(binary), "check", f"examples/{name}.rs"], directory / f"{name}.log")
    entry = {"case": name, "expectedExit": expected, "exit": code, "wallSeconds": elapsed}
    try:
        path, report = validate_report(output, name, expected, code, binary)
        if compiler is not None and report.get("rustc", "").strip() != compiler.strip():
            raise ValueError("report rustc does not match the selected toolchain")
        if compiler_path is not None and report.get("compilerPath") != str(compiler_path):
            raise ValueError("report compiler path does not match the resolved toolchain")
        entry["toolchain"] = toolchain or os.environ.get("RUSTUP_TOOLCHAIN", "caller selection")
        (directory / f"{name}.json").write_text(json.dumps(report, indent=2) + "\n")
        entry["report"] = str(path)
        if expected == 1:
            artifact = path.parent
            for filename in ("replay.rs", "counterexample.smt2"):
                if not (artifact / filename).is_file() or not (artifact / filename).stat().st_size:
                    raise ValueError(f"missing/empty required {filename}")
            compile_code, _, _ = execute([report["compilerPath"], "--edition=2024", "-C", "opt-level=1",
                                     "-C", "overflow-checks=yes", str(artifact / "replay.rs"),
                                     "-o", str(artifact / "replay")], directory / f"{name}-replay-build.log")
            entry["nativeReplayBuildExit"] = compile_code
            if compile_code != 0:
                raise ValueError("native replay did not compile")
            replay_code, replay_output, _ = execute([str(artifact / "replay")], directory / f"{name}-native.log")
            entry["nativeReplayExit"] = replay_code
            if replay_code != 0 or not CONFIRMATIONS.intersection(replay_output.splitlines()):
                raise ValueError("native replay did not confirm the counterexample")
            smt_code, smt_output, _ = execute(["z3", str(artifact / "counterexample.smt2")], directory / f"{name}-solver-replay.log")
            entry["solverReplayExit"] = smt_code
            lines = smt_output.splitlines()
            statuses = [line.strip() for line in lines
                        if line.strip() in {"sat", "unsat", "unknown"}]
            if (smt_code != 0 or not lines or lines[0] != "sat"
                    or statuses != ["sat"]
                    or any("error" in line.lower() for line in lines)):
                raise ValueError("solver replay must return exactly one status, sat, without errors")
    except (OSError, ValueError, KeyError, TypeError) as error:
        entry["error"] = str(error)
        (directory / f"{name}-validation.log").write_text(str(error) + "\n")
    return entry


def resolve_compiler(toolchain):
    env = dict(os.environ, RUSTUP_TOOLCHAIN=toolchain)
    def rustc(program, *args):
        return subprocess.check_output([str(program), *args], cwd=ROOT, env=env, text=True).strip()
    selected = Path(rustc("rustc", "--print", "sysroot"))
    compiler = (selected / "bin" / "rustc").resolve(strict=True)
    version = rustc(compiler, "-vV")
    sysroot = Path(rustc(compiler, "--print", "sysroot"))
    host = next(line[5:].strip() for line in version.splitlines() if line.startswith("host:"))
    tools = sysroot / "lib/rustlib" / host / "bin"
    return version, tools, compiler


def compiler_info(toolchain):
    version, tools, compiler = resolve_compiler(toolchain)
    llvm = next(line[13:].strip() for line in version.splitlines() if line.startswith("LLVM version:"))
    major = llvm.split(".")[0]
    paths = [Path(p) for p in os.environ.get("PATH", "").split(os.pathsep)]
    for name in ("llvm-ar", "llvm-objcopy", "llvm-link", "llvm-dis"):
        candidates = [tools / name, *(p / f"{name}-{major}" for p in paths), *(p / name for p in paths)]
        rejected = []
        for candidate in candidates:
            if not candidate.is_file():
                continue
            try:
                text = subprocess.check_output([str(candidate), "--version"], text=True, stderr=subprocess.STDOUT)
                matches = [re.search(r"version ([0-9]+)(?:\.[^\s]*)?", line) for line in text.splitlines() if "LLVM" in line]
                if any(match and match[1] == major for match in matches):
                    break
                rejected.append(f"{candidate}: {text.strip()}")
            except (OSError, subprocess.CalledProcessError) as error:
                rejected.append(f"{candidate}: {error}")
        else:
            raise ValueError(f"no matching {name} for LLVM {llvm}: " + ("; ".join(rejected) or "tool missing")
                             + f". Fix: rustup component add llvm-tools --toolchain {toolchain}")
    return version, tools, compiler


def installed_toolchains():
    output = subprocess.check_output(["rustup", "toolchain", "list"], text=True, cwd=ROOT)
    eligible = []
    for line in output.splitlines():
        name = line.split()[0]
        _, tools, _ = resolve_compiler(name)
        if all((tools / tool).is_file() for tool in ("llvm-ar", "llvm-objcopy", "llvm-link", "llvm-dis")):
            eligible.append(name)
        else:
            print(f"skipping {name}: llvm-tools not installed", flush=True)
    if not eligible:
        raise ValueError("no installed toolchain has llvm-tools")
    return eligible


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/phage")
    parser.add_argument("--standalone", action="store_true", help="only fixtures contained in this checkout")
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--toolchain", help="rustup toolchain for checked code and native replay")
    selection.add_argument("--all-toolchains", action="store_true", help="every installed rustup toolchain with llvm-tools")
    args = parser.parse_args(argv)
    directory = ROOT / "evidence" / f"run-{time.time_ns()}"
    directory.mkdir(parents=True)
    try:
        toolchains = installed_toolchains() if args.all_toolchains else [args.toolchain]
        compilers = {t: compiler_info(t) for t in toolchains if t is not None}
    except (OSError, ValueError, subprocess.CalledProcessError, StopIteration) as error:
        (directory / "toolchain-error.log").write_text(str(error) + "\n")
        print(f"FAILED: toolchain discovery: {error}\nevidence: {directory}", flush=True)
        return 1
    results = []
    for toolchain in toolchains:
        case_directory = directory
        if toolchain is not None:
            # Do not interpret an arbitrary toolchain argument as an output path.
            case_directory = directory / hashlib.sha256(toolchain.encode()).hexdigest()[:16]
            case_directory.mkdir()
            (case_directory / "toolchain.txt").write_text(toolchain + "\n" + compilers[toolchain][0] + "\n" + str(compilers[toolchain][2]) + "\n")
        for name, expected in selected_cases(args.standalone):
            entry = check_case(name, expected, ROOT / args.binary, case_directory,
                               toolchain, compilers[toolchain][0] if toolchain else None,
                               compilers[toolchain][2] if toolchain else None)
            entry["toolchain"] = toolchain or os.environ.get("RUSTUP_TOOLCHAIN", "caller selection")
            results.append(entry)
            (directory / "RESULT.json").write_text(json.dumps(results, indent=2) + "\n")
            print(f"{entry['toolchain']} / {name}: exit={entry['exit']} expected={expected}, {entry['wallSeconds']:.3f}s"
                  + (f", FAILED: {entry['error']}" if "error" in entry else ""), flush=True)
    print(f"evidence: {directory}", flush=True)
    return int(not results or any("error" in entry for entry in results))


if __name__ == "__main__":
    raise SystemExit(main())
