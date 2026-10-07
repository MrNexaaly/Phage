"""Fail-closed controls for the examples runner (stdlib only).

Fixtures use the inspected schema-4 LLVM report and real report: announcement.
Subprocess outcomes are mocked here; CI also runs real standalone examples.
"""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("examples_runner", Path(__file__).resolve().parents[1] / "tools/examples.py")
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.root_patch = patch.object(runner, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.source = self.root / "examples/overflow.rs"
        self.source.parent.mkdir()
        self.source.write_text("pub fn phage_target(x: u8) -> bool { x + 1 > x }\n")
        self.binary = self.root / "target/release/phage"
        self.binary.parent.mkdir(parents=True)
        self.binary.write_bytes(b"verifier fixture")
        self.compiler = self.root / "rustc"
        self.compiler.write_bytes(b"resolved compiler fixture")
        self.artifact = self.root / "results/test"
        self.artifact.mkdir(parents=True)
        (self.artifact / "artifact.ll").write_text("; compiled LLVM fixture\n")
        (self.artifact / "replay.rs").write_text("fn main() {}\n")
        (self.artifact / "counterexample.smt2").write_text("(check-sat)\n")
        self.snapshot = self.artifact / "sources" / self.source.relative_to("/")
        self.snapshot.parent.mkdir(parents=True)
        self.snapshot.write_bytes(self.source.read_bytes())
        self.report_path = self.artifact / "RESULT.json"
        # Fields used by validation match the observed real schema-4 report.
        self.report = {
            "schema": 4, "engine": "llvm", "compilerPath": str(self.compiler), "status": "counterexample",
            "source": str(self.source), "function": "phage_target",
            "artifactSha256": runner.digest(self.artifact / "artifact.ll"),
            "llvmSha256": runner.digest(self.artifact / "artifact.ll"),
            "fingerprints": [{"path": str(p), "sha256": runner.digest(p)}
                             for p in (self.source, self.binary, self.compiler)],
        }
        self.save_report()
        self.output = "run: ignored-path\nreport: results/test/RESULT.json\n"
        self.exit = 1
        self.compile_exit = 0
        self.native_exit = 0
        self.native_output = "confirmed: Rust panicked\n"
        self.solver_exit = 0
        self.solver_output = "sat\n((input0 #xff))\n"
        self.calls = []

    def save_report(self):
        self.report_path.write_text(json.dumps(self.report))

    def fake_run(self, command, logfile, timeout=180, env=None):
        self.calls.append(command)
        if env is not None:
            self.assertEqual(env["RUSTUP_TOOLCHAIN"], "stable")
        if command[0] == str(self.binary):
            code, output = self.exit, self.output
        elif command[0] == str(self.compiler):
            code, output = self.compile_exit, "build output"
        elif command[0] == "z3":
            code, output = self.solver_exit, self.solver_output
        else:
            code, output = self.native_exit, self.native_output
        logfile.write_text(output)
        return code, output, 0.01

    def invoke(self, expected_success):
        with patch.object(runner, "CASES", [("overflow", 1)]), patch.object(runner, "run", self.fake_run), contextlib.redirect_stdout(io.StringIO()):
            code = runner.main(["--standalone"])
        self.assertEqual(code, 0 if expected_success else 1)
        evidence = sorted((self.root / "evidence").glob("run-*"))[-1]
        entry = json.loads((evidence / "RESULT.json").read_text())[0]
        self.assertEqual("error" not in entry, expected_success)
        if not expected_success:
            self.assertTrue((evidence / "overflow-validation.log").is_file())
        return entry

    def test_valid_report_and_both_replays(self):
        entry = self.invoke(True)
        self.assertEqual(entry["nativeReplayExit"], 0)
        self.assertEqual(entry["solverReplayExit"], 0)
        self.assertEqual(len(self.calls), 4)

    def test_absolute_report_without_run_announcement(self):
        self.output = f"report: {self.report_path}\n"
        self.invoke(True)

    def test_missing_or_ambiguous_announcement(self):
        for output in ("run: results/test\n", "report: \n", self.output + self.output):
            with self.subTest(output=output):
                self.output = output
                self.invoke(False)

    def test_missing_report(self):
        self.report_path.unlink()
        self.invoke(False)

    def test_malformed_or_nonobject_report(self):
        for value in ("{broken", "[]", "null"):
            with self.subTest(value=value):
                self.report_path.write_text(value)
                self.invoke(False)

    def test_wrong_schema_status_source_function_or_engine(self):
        for key, wrong in (("schema", 3), ("status", "proved"), ("source", "/wrong.rs"),
                           ("function", "other"), ("engine", "mir")):
            with self.subTest(key=key):
                before = self.report[key]
                self.report[key] = wrong
                self.save_report()
                self.invoke(False)
                self.report[key] = before

    def test_wrong_exit_and_timeout(self):
        for code in (0, 2, None):
            with self.subTest(code=code):
                self.exit = code
                self.invoke(False)

    def test_missing_or_invalid_fingerprints(self):
        good = self.report["fingerprints"]
        for wrong in (None, [], good[:1], good[1:], [None], good + good,
                      [{"path": "relative.rs", "sha256": "bad"}]):
            with self.subTest(wrong=wrong):
                self.report["fingerprints"] = wrong
                self.save_report()
                self.invoke(False)

    def test_changed_source_or_binary(self):
        for path in (self.source, self.binary):
            with self.subTest(path=path):
                before = path.read_bytes()
                path.write_bytes(b"changed")
                self.invoke(False)
                path.write_bytes(before)

    def test_missing_or_changed_snapshot(self):
        self.snapshot.write_text("changed snapshot")
        self.invoke(False)
        self.snapshot.unlink()
        self.invoke(False)

    def test_missing_or_changed_llvm_artifact(self):
        path = self.artifact / "artifact.ll"
        path.write_text("changed artifact")
        self.invoke(False)
        path.unlink()
        self.invoke(False)

    def test_wrong_artifact_hash_field(self):
        self.report["llvmSha256"] = "0" * 64
        self.save_report()
        self.invoke(False)

    def test_missing_or_empty_replays(self):
        for name in ("replay.rs", "counterexample.smt2"):
            with self.subTest(name=name):
                path = self.artifact / name
                before = path.read_bytes()
                path.write_bytes(b"")
                self.invoke(False)
                path.unlink()
                self.invoke(False)
                path.write_bytes(before)

    def test_replay_compile_failure(self):
        self.compile_exit = 1
        self.invoke(False)
        self.assertEqual(len(self.calls), 2)

    def test_native_failure_timeout_or_missing_confirmation(self):
        for code, output in ((2, "input did not reproduce"), (None, "timeout"), (0, "")):
            with self.subTest(code=code):
                self.native_exit, self.native_output = code, output
                self.invoke(False)

    def test_solver_unsat_unknown_error_or_timeout(self):
        for code, output in ((0, "unsat\n"), (0, "unknown\n"), (1, "sat\n"),
                             (None, "timeout"), (0, 'sat\n(error "bad model")\n')):
            with self.subTest(code=code, output=output):
                self.solver_exit, self.solver_output = code, output
                self.invoke(False)

    def test_solver_rejects_multiple_statuses(self):
        for output in ("sat\nunsat\n", "sat\nunknown\n", "sat\nsat\n",
                       "sat\n((input0 #xff))\n  unknown  \n"):
            with self.subTest(output=output):
                self.solver_output = output
                self.invoke(False)

    def test_proved_and_unknown_require_report_but_not_replay(self):
        for expected, status in ((0, "proved"), (2, "unknown")):
            with self.subTest(status=status), patch.object(runner, "run", self.fake_run):
                self.report["status"] = status
                self.save_report()
                self.exit = expected
                entry = runner.check_case("overflow", expected, self.binary, self.root)
                self.assertNotIn("error", entry)
                self.assertNotIn("nativeReplayExit", entry)
                self.report_path.unlink()
                entry = runner.check_case("overflow", expected, self.binary, self.root)
                self.assertIn("error", entry)
                self.save_report()

    def test_selection_preserves_local_hpack_and_all_outcomes(self):
        standalone = runner.selected_cases(True)
        self.assertTrue(standalone)
        self.assertIn(("nexagate-hpack-before", 1), standalone)
        self.assertEqual({code for _, code in standalone}, {0, 1, 2})
        self.assertEqual(set(runner.CASES) - set(standalone),
                         {("nexagate-quic", 0), ("nexagate-hpack-after", 0)})
        self.assertEqual(runner.selected_cases(False), runner.CASES)

    def test_explicit_toolchain_applies_to_check_and_native_replay(self):
        compiler = "release: 1.99.0\nLLVM version: 23.1.1"
        self.report["rustc"] = compiler
        self.save_report()
        with patch.object(runner, "compiler_info", return_value=(compiler, Path("unused"), self.compiler)), patch.object(runner, "CASES", [("overflow", 1)]), patch.object(runner, "run", self.fake_run), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.main(["--toolchain", "stable"]), 0)
        self.assertEqual(len(self.calls), 4)
        self.assertEqual(self.calls[1][0], str(self.compiler))
        self.report["rustc"] = "release: 1.94.1\nLLVM version: 21.1.8"
        self.save_report()
        with patch.object(runner, "run", self.fake_run):
            entry = runner.check_case("overflow", 1, self.binary, self.root, "stable", compiler)
        self.assertIn("selected toolchain", entry["error"])

    def test_all_toolchains_filters_missing_tools_and_rejects_empty_set(self):
        tools = self.root / "llvm-bin"
        tools.mkdir()
        for tool in ("llvm-ar", "llvm-objcopy", "llvm-link", "llvm-dis"):
            (tools / tool).touch()
        def info(name):
            return "version", tools if name == "stable-host" else self.root / "missing", self.compiler
        with patch.object(runner.subprocess, "check_output", return_value="stable-host (default)\nold-host\n"), patch.object(runner, "resolve_compiler", side_effect=info), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.installed_toolchains(), ["stable-host"])
        (tools / "llvm-link").unlink()
        with patch.object(runner.subprocess, "check_output", return_value="stable-host\n"), patch.object(runner, "resolve_compiler", side_effect=info), contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(ValueError, "no installed toolchain"):
                runner.installed_toolchains()

    def test_toolchain_discovery_errors_fail_closed_with_retained_log(self):
        with patch.object(runner, "compiler_info", side_effect=FileNotFoundError("rustc unavailable")), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.main(["--toolchain", "missing"]), 1)
        evidence = sorted((self.root / "evidence").glob("run-*"))[-1]
        self.assertIn("rustc unavailable", (evidence / "toolchain-error.log").read_text())

    def test_preflight_checks_real_tool_versions_and_fails_with_fix(self):
        tools = self.root / "llvm-tools"
        tools.mkdir()
        version = "release: 1.99.0\nLLVM version: 23.1.1"
        for name in ("llvm-ar", "llvm-objcopy", "llvm-link", "llvm-dis"):
            path = tools / name
            path.write_text("#!/bin/sh\nprintf '%s\\n' 'LLVM version 23.1.1'\n")
            path.chmod(0o755)
        with patch.object(runner, "resolve_compiler", return_value=(version, tools, self.compiler)), patch.dict(os.environ, {"PATH": str(tools)}):
            self.assertEqual(runner.compiler_info("stable"), (version, tools, self.compiler))
            (tools / "llvm-link").write_text("#!/bin/sh\nprintf '%s\\n' 'LLVM version 22.1.8'\n")
            with self.assertRaisesRegex(ValueError, "rustup component add llvm-tools --toolchain stable"):
                runner.compiler_info("stable")

    def test_native_replay_refuses_missing_or_unfingerprinted_compiler(self):
        for path in ("", "rustc", str(self.root / "unfingerprinted-rustc")):
            self.report["compilerPath"] = path
            self.save_report()
            self.invoke(False)

    def test_run_records_launch_errors_and_timeout(self):
        for error in (FileNotFoundError("missing command"),
                      subprocess.TimeoutExpired(["fixture"], 1, output=b"partial", stderr=b"error")):
            with self.subTest(error=error), patch.object(runner.subprocess, "run", side_effect=error):
                log = self.root / "error.log"
                code, output, _ = runner.run(["fixture"], log)
                self.assertIsNone(code)
                self.assertEqual(output, log.read_text())
                self.assertTrue(output)


if __name__ == "__main__":
    unittest.main()
