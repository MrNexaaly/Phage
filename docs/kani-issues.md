# What Phage learned from Kani issues

Reviewed on 2026-09-30. The saved [open issue index](../evidence/improvements/kani-issues/open-index.json)
contains 460 issue titles and metadata from GitHub's API, excluding pull
requests. We examined the selected issue bodies below. This is not a claim
that every listed issue has been reproduced or fixed.

Issue-list pages can be cached. Direct issue pages and API state showed that
4874 and 4867 are already closed. We keep their cases as regressions, and
compare against the installed Kani 0.68.0 separately from upstream main.

## Current protections

| Upstream report | State checked | Phage change or control | Coverage boundary |
| --- | --- | --- | --- |
| [4874: assertions in expression positions under strict warnings](https://github.com/model-checking/kani/issues/4874) | Closed | Standard rustc assertions compile without replacement; positive constant/match forms prove, and the failing assertion is found and replayed. | Installed Kani 0.68.0 fails these shared inputs at compilation; this does not claim the merged upstream fix is broken. |
| [1423: missing functions become apparent only through a failing assertion](https://github.com/model-checking/kani/issues/1423) | Open | An encountered unresolved call is `unknown`, tagged `unsupportedCall`, with its symbol and original Rust location. | General function calls still need implementation; we do not stub them or invent an application counterexample. |
| [3012: lost assertion metadata](https://github.com/model-checking/kani/issues/3012) | Open | Stable result classes and Rust file/line/column survive into schema 2 JSON and CLI output. The negative assertion control checks its exact source line. | This protects our reporting pipeline; Kani's experimental coverage/irep path is not part of Phage. |
| [3451: driver panic when a process has no numeric exit code](https://github.com/model-checking/kani/issues/3451) | Open | A real Z3 child is killed in a regression. Its signal becomes a solver error, which the CLI reports as unknown. | This is an analogous process-failure control, not a reproduction of that older Kani version. |
| [4877: indistinguishable skip causes](https://github.com/model-checking/kani/issues/4877) | Open | Separate `unsupportedCall`, `unsupportedType`, `blockLimit`, `stateLimit`, `solverUnknown` and `solverExit` categories. CLI controls assert distinct reasons. | Automatic trait-implementor search does not exist yet; its cause analysis remains future work. |
| [4876: non-usize const generics](https://github.com/model-checking/kani/issues/4876) | Open | Explicit u8/bool const-generic instantiations prove through ordinary Rust compilation; both tools pass the identical property. | Automatic generic harness generation is not implemented. Passing an explicit property does not solve autoharness discovery. |
| [4867: compiler crash on pointer SIMD](https://github.com/model-checking/kani/issues/4867) | Closed | A valid LLVM pointer-vector signature gets a structured unsupported-type unknown, without crashing. | Pointer SIMD semantics and nightly Rust portable_simd are not implemented. |
| [4720: symbolic index/carry-chain performance and lack of instrumentation](https://github.com/model-checking/kani/issues/4720) | Open | Shared path-prefix assertions and proved pointer simplifications reduce repeated work. Solver counts/time and the longest query's context, source and standalone SMT are saved. | The reported 34-limb floating-point accumulator remains outside our supported subset; no claim to solve its multi-hour proof. |

## Reproduce the controls

```sh
cargo test
cargo build --release
python3 tools/issues.py
python3 tools/examples.py
python3 tools/compare_kani.py --cases quic hpack assert-forms false-assert const-generics --repeats 3
```

The issue runner checks expected exit codes and reason codes. Missing calls,
unsupported vector types, block limits and state limits must return unknown.
The false assertion must reach verification and fail at its own Rust source
line; a compiler failure does not satisfy that negative control.

The example runner replays candidate inputs in native Rust and independently
reruns their SMT obligations. New replay templates use captured source
snapshots. Earlier attempts, including the first comparison's generic
`unknown-error` classification for the macro compile failures, are retained.

See [measured improvements](../evidence/improvements/SUMMARY.md) and the
[codebase map](../CODEBASE_MAP.md) for ownership and extension rules.

## Next capabilities that still need implementation

1. Parse and check ordinary direct function calls, then recursion with
   explicit depth limits and sound per-call environments.
2. Generate harnesses for supported Rust signatures and explicit generic
   choices; explain each rejected signature or trait requirement.
3. Broaden array/aggregate layouts and preserve aliasing, lifetime and
   initialization obligations. Do not equate a pointer vector with arbitrary
   integer addresses merely to make a proof pass.
4. Validate source-level undefined behavior with a frontend that preserves
   it. The current optimized LLVM proof boundary remains narrower than
   verification of every Rust source execution.
5. Add complete symbolic-array/carry benchmarks before claiming progress on
   the larger performance reports.
