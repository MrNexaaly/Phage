# RuHealth first engine results

The independent interpreter is working on two real Nexagate production
functions. These results concern compiled LLVM and the stated input domains.
They do not establish production readiness or superiority over Kani.

## Fresh final run

Full logs and reports: [run-1790769095646275658](../run-1790769095646275658/RESULT.json).

| Case | Result | Full CLI elapsed | Native replay |
| --- | --- | --- | --- |
| safe-add | proved | 0.068 s | not applicable |
| overflow | counterexample | 0.065 s | confirmed |
| nexagate-quic | proved | 2.228 s | not applicable |
| nexagate-hpack-before | counterexample | 62.906 s | confirmed |
| nexagate-hpack-after | proved | 10.820 s | not applicable |
| unsupported | unknown | 0.071 s | not applicable |

All six expected exit codes matched. Both counterexamples were replayed in
native Rust and their retained SMT queries were independently rerun with Z3.
The original HPACK panicked at its shift; the fixed code was also checked
with the original counterexample in `replay-after.rs`.

Formatting and Clippy passed. All 30 semantic controls passed, including
poison, immediate zero division, uninitialized reads, simultaneous phi,
conditional pointers, reachable limits, solver exhaustion and unsupported
calls. The engine forbids unsafe code and has no Rust crate dependencies.

## Domains

- QUIC: all 8 symbolic bytes; lengths 0 through 8. Matches the Kani read
  property: existence/truncation, used width and 62-bit value limit.
- HPACK: all 12 symbolic bytes; lengths 0 through 12; prefix widths 1 through
  8. Matches the Kani integer property: no panic and success value/used bounds.
- Block visit bound 32, state limit 10,000, per-query timeout 15,000 ms.
  A feasible path exceeding a limit would prevent proof.

The current QUIC and fixed HPACK source hashes match Kani's saved final run.
The old HPACK fixture hash matches the frozen pre-fix source. Per-case JSON
records all hashes, compiler flags, tool versions and the engine binary hash.

## Kani comparison

Kani's earlier final run proved seven properties. Its solver time was 1.175
seconds for QUIC read and 1.098 seconds for fixed HPACK. RuHealth times above
include frontend compilation, execution, hashing and artifact collection.
These different timing scopes and compiler versions do not support a speed
ranking. RuHealth currently has substantially narrower instruction and Rust
feature coverage. Both tools found the same native-confirmed HPACK defect.

Kani evidence: `../../../Nexagate/verification/results/20260930-final/RESULT.json`.
RuHealth uses rustc 1.94.1 / LLVM 21.1.8 and Z3 4.16.0. Kani used its own
nightly frontend and CBMC 6.11.0. Neither result proves Nexagate's whole server.

## Earlier attempts

[ATTEMPTS.md](ATTEMPTS.md) retains earlier unknown results, rejected native
replay build, failed resource-control expectation and semantic review changes.
Earlier artifact directories remain present; the final run supersedes their
claims for the current engine. No source defect is inferred solely from a
solver model: native reproduction is recorded separately.
