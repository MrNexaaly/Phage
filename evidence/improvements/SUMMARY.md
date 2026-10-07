# RuHealth improvements and matched Kani comparison

RuHealth now beats the installed Kani 0.68.0 on the three shared-source
Nexagate checks below. This measures complete CLI execution after a warmup;
it does not claim superiority across Rust language features or verifier
soundness. The CPU was busy with other host work, which is recorded in the
result; these are measurements on this machine under that load.

## Same CPU, repeated full CLI runs

Both tools and their child processes were pinned to CPU 0. There was one
retained unmeasured warmup, then three measured runs per tool/case, with
alternating order. All proof and counterexample outcomes agreed on every run.

Raw data: [comparison-1790775835840327970](../comparison-1790775835840327970/RESULT.json).

| Same property | RuHealth median | Kani median | RuHealth speedup |
| --- | --- | --- | --- |
| QUIC reader | 0.285 s | 8.895 s | 31.2× |
| Fixed HPACK | 1.561 s | 6.078 s | 3.9× |
| Old HPACK defect | 2.326 s | 8.268 s | 3.6× |

A separate five-repeat run without CPU pinning also showed wins on all
three checks: [five-repeat data](../comparison-1790775301722298784/RESULT.json).
It ran on the shared machine under heavy CPU contention; compare its results
within that run rather than combining its times with quieter earlier runs.
The pinned run records load averages and CPU pressure before/after.

## Property and input equivalence

The Kani comparison workspace includes the exact same `examples/*.rs` files
that RuHealth compiles, rather than a rewritten specification. QUIC uses all
8 symbolic bytes and a symbolic u8 length, with lengths above 8 excluded by
that shared property. HPACK uses all 12 symbolic bytes, a symbolic u8 length
and prefix, with lengths 0–12 and prefixes 1–8 checked. There are no stubs,
removed assertions or assumed exploration-limit success.

Both configurations use bound 32 (per-block visits in RuHealth, unwind bound
in Kani). Both complete these bounded input domains. Source and engine hashes,
commands and tool versions are recorded. RuHealth uses rustc 1.94.1 / LLVM
21.1.8 / Z3 4.16.0. Kani uses its released nightly frontend and CBMC 6.11.0.
Full-process timing includes both tools' frontend work; solver wall time and
Kani's reported verification time are also available but have different
instrumentation boundaries.

## What changed

- Reuse SMT path prefixes through push/pop instead of reasserting an entire
  path at every obligation. SSA declarations remain global during backtracking.
- Keep concrete definedness literal and skip literal-false obligations as
  unsatisfiable. Feasible-path checks are still required.
- Compare same-allocation pointer equality through offsets; distribute
  conditional pointer equality; fold null checks only for pointers whose
  defined value is within a nonnull, nonwrapping allocation.
- Remove the redundant absolute-address no-wrap formula only when inbounds,
  allocation extent and unsigned offset no-wrap imply it. A Z3 regression
  proves this implication for all 64-bit base/size/offset/index values. A GEP
  without inbounds keeps the full obligation.
- Preserve poison obligations and add negative controls for a wrapping pointer
  that can be null and a poisoned inbounds pointer comparison.
- Add schema 2 diagnostic codes and original Rust file/line/column, explicit
  solver-exit information, counters, unknown reasons, longest-query context
  and standalone `slowest-query.smt2`.
- Generate native replay against captured source snapshots. Relocating a run
  outside the workspace still reproduces the HPACK counterexample.

The earlier [three-repeat comparison](../comparison-1790773948397838415/RESULT.json)
showed HPACK at 11.107 s in our retained original engine, 4.305 s after the
first improvements, and 2.372 s in Kani. HPACK was still a loss then. The final
pointer changes produced a standalone 0.663 s proof, followed by the repeated
wins above. We keep every earlier log rather than replace that failed target.

## Qualification

- 41 Rust semantic, metadata, diagnostic and solver controls passed.
- Clippy and formatting passed; the package still forbids unsafe engine code
  and has zero Rust crate dependencies.
- All nine example exit codes matched; three candidate inputs reproduced in
  native Rust and their SMT queries independently returned sat.
- All seven issue-inspired CLI controls passed, including exact assertion
  source line, named missing function, unsupported pointer vector and distinct
  block/state limit reasons.
- All nine longest-query artifacts independently replayed with Z3:
  [profile replay](profile-replay-final.json).
- Captured sources reproduced the old HPACK panic after moving the run:
  [relocated replay](relocated-replay-final.json).

Final example data: [run-1790774911414084951](../run-1790774911414084951/RESULT.json).
Issue controls: [issues-1790774910978290487](../issues-1790774910978290487/RESULT.json).

## Issues and remaining coverage

The [Kani issue review](../../docs/kani-issues.md) records exact upstream state
and our corresponding controls. Standard strict-warning assertion forms pass
here and fail to compile in installed Kani 0.68.0. Issue 4874 is already closed
upstream; the result is not a claim against its merged fix. Explicit u8/bool
const-generic properties pass both tools; automatic harness discovery remains
unimplemented in RuHealth.

General function calls, heap/aggregate ownership, async/concurrency and a
frontend that preserves all source-level undefined behavior remain coverage
work. These measured wins establish a competitive parser-verification slice.
The [codebase map](../../CODEBASE_MAP.md) explains how to extend it.

## Five-repeat check without pinning

The retained five-repeat run also passed every expected outcome:

| Case | RuHealth median | Kani median | Speedup |
| --- | --- | --- | --- |
| QUIC | 0.651 s | 12.115 s | 18.6x |
| Fixed HPACK | 2.361 s | 6.815 s | 2.9x |
| Old HPACK defect | 3.188 s | 8.063 s | 2.5x |

Pinning was added after observing substantial host CPU contention. It does
not reserve the core exclusively; both tools share that same core with other
host work. The repeated wins are real for these recorded runs, and neither
set of elapsed times is a claim about an unloaded machine.
