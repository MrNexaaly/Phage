# Phage user guide

Run commands from the repository root. The [project overview](../README.md) is the short introduction.

Phage checks small Rust functions for bugs by exploring symbolic inputs.
It has its own verification engine. rustc compiles Rust; Z3 solves the math
constraints. It does not run Kani or CBMC.

It verifies sequential Rust code including calls, the standard library's
own code (Vec, String, Box, VecDeque, HashMap, Rc, sorting, trait objects and
more), heap and pointer memory, statics, atomics and SIMD integer vectors.
Measured held-out sets found no wrong verdicts and some wins over Kani;
those results do not establish general verifier correctness. It decides some
HashMap/HashSet properties where Kani times out. It handles `f32`
and `f64` (IEEE-754 through the solver's floating-point theory). It is still
young. See the measured comparison with Kani in
[the research notes](production-readiness-research.md).

Phage was called RuHealth until 2026-10-01. The old entry name
`ruhealth_target` and `RUHEALTH_*` environment variables still work.

## Run it

Requirements: Rust, Z3 and `sha256sum` on Linux x86_64. The Rust package has
no crate dependencies. Standard library bodies need matching LLVM tools:
`rustup component add llvm-tools --toolchain <name>`. Phage first uses the
active rustc's `lib/rustlib/<host>/bin/` tools, then `llvm-<tool>-NN` on PATH,
then a bare LLVM tool only when its reported LLVM major matches rustc.
Missing or mismatched tools give an actionable unknown, never a wrong cache. `bitwuzla` and `cvc5` on the
PATH join the solver race: Bitwuzla decides hash-heavy bit-vector queries
several times faster, cvc5 nonlinear queries Z3 cannot. Python 3 is needed
only for the example runner.

```sh
cargo build --release
target/release/phage check examples/safe-add.rs
target/release/phage check examples/nexagate-quic.rs
```

| Result | Meaning | Exit |
| --- | --- | --- |
| `proved` | All reachable paths completed successfully within the limits, using the supported LLVM model. | 0 |
| `counterexample` | A false property, panic, a call that never returns, undefined behavior (poison use, invalid memory access, violated call contract). Confirm source defects with native replay. | 1 |
| `unknown` | Unsupported code, a reachable limit, solver timeout or analysis error prevented a conclusion. | 2 |

## Use it as a gate

Phage is an additional bounded check, not a replacement for tests, fuzzing
or review. A `proved` result covers the named property and retained artifact,
under the report's model assumptions. It is not a whole-program safety claim.

- Only exit **0** is a completed proof. Exit **1** and **2** must block a
  proof-required gate; do not treat `unknown` as a pass.
- Require a readable `RESULT.json` with `status: "proved"`; retain the artifact,
  source hashes, compiler flags, limits and versions. Reject missing evidence.
- Review every recorded assumption against your deployment. Allocation success,
  one thread and fresh statics are model limits, not promises about a server.
- LLVM call argument restrictions are never dropped to obtain a proof. A
  possibly violated `nonnull`, `align` or `range` restriction without `noundef`
  yields `unknown` until its poison behavior can be modeled. With `noundef`,
  a reachable violation is a counterexample. Byte-operation argument attributes are checked even when the length is zero.
  Alignment without noundef on a zero-length operation that may poison an
  argument remains unknown; concrete nonzero operations check alignment with
  their access obligations.
- Confirm source bugs using native replay where possible. A compiler-artifact
  defect or address-dependent counterexample is not automatically a source bug.

The [CI workflow](../.github/workflows/ci.yml) checks a pinned Rust toolchain,
formatting, strict lint, locked build/tests, runner negative controls and
self-contained examples with native and solver replay. Run the portable subset
with `python3 tools/examples.py --standalone`. Production examples that import
Nexagate remain separate; they need that sibling checkout. Hosted CI must be
observed passing before relying on it. `rust-toolchain.toml` pins the compiler
used to build Phage to 1.94.1. Checked code uses the caller's rustc selection:
`RUSTUP_TOOLCHAIN` takes precedence, otherwise rustup uses the caller's working
directory (including any rust-toolchain file) and default toolchain. To check
with stable from this checkout, set `RUSTUP_TOOLCHAIN=stable`; the build pin is
not a hard-coded checked-code frontend. CI checks the standalone examples on
both the pinned compiler and current stable. The solver wheel is pinned to Z3 4.16.0. The hosted OS image
and package delivery are not fully hermetic, so this is not a bit-for-bit
reproducible build.

Locally verified on 2026-10-05: all 11 known-verdict examples on rustc
1.94.1 / LLVM 21.1.8 and stable rustc 1.99.0 / LLVM 23.1.1, with Z3 4.16.0.
This covers the retained bounded artifacts, not all code on those toolchains.
Before compilation Phage resolves one absolute rustc in the selected sysroot.
Its version, executable path, sysroot and binary fingerprint are recorded;
compilation, library discovery and native replay use that same compiler.
The cache identity includes full rustc/LLVM versions, compiler path, sysroot,
rlib metadata and SHA-256. A manifest records the exact input identity and
SHA-256 of the index/modules. Cache corruption is unknown; files are checked
on open and the exact module bytes are checked again before parsing. Cache
staging directories are exclusively created. Both old size-plus-pointer and LLVM 22+ pointer-only whole-object
lifetime markers are supported. Only lifetime.start makes a stack object initially dead; end-only objects
start alive. Allocations with disjoint lifetimes may share an address. Dead stack
loads yield poison, checked at an observed use; dead stores are undefined
behavior. Poison markers have no effect, and markers can never resurrect a
heap allocation or a returned stack frame. Exact local/caller associations
are classified before allocation; unresolved aliases stay unknown.
`llvm.assume` nonnull bundles are checked as obligations; ignore bundles are
no-ops. Other bundle kinds
stay unknown. Unrecognized constructs/signatures in compiled Rust diagnostics
include rustc/LLVM versions and a model-drift hint; known unsupported features
(such as unbounded symbolic heap sizes and frem) do not receive that hint;
for direct `.ll` inputs the producer is unknown and is labeled accordingly.
See the reviewed toolchain and lifetime evidence (`../evidence/round2-work/SUMMARY.md`, retained reference; artifact not in this snapshot).
The first-round report is retained as historical evidence and is superseded
by the soundness review corrections.

```sh
python3 tools/examples.py --toolchain 1.94.1
python3 tools/examples.py --toolchain stable
python3 tools/examples.py --standalone --all-toolchains
```

`--all-toolchains` runs every installed rustup toolchain with llvm-tools;
`--toolchain` preflights all four required LLVM tools with actionable failures;
native counterexample replay uses the report’s absolute resolved compiler. Every attempt
is retained, including missing-toolchain errors and unexpected verdicts.

## Write a property

Write an exported function that takes integers or bounded byte references and returns `bool`.
Returning `true` means the property holds. All inputs are symbolic.

```rust
#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    let bigger = u16::from(x) + 1;
    bigger > u16::from(x)
}
```

Save it as `property.rs`, then run:

```sh
target/release/phage check property.rs
```

To check existing code, include its module and call it from your property.
[The QUIC example](../examples/nexagate-quic.rs) includes Nexagate's real parser.
To restrict the input domain, return true for excluded inputs before calling
the code. Excluded inputs are not checked against the intended property.

Bounded byte references (`&[u8; N]` and `&mut [u8; N]`) use a fresh,
fully initialized symbolic byte array with the compiled `dereferenceable(N)`
extent and declared alignment (1 when omitted). The object is non-null,
alive for the entry call, and disjoint from every other modeled object.
`readonly` forbids writes and `writeonly` forbids reads. Those permissions
follow derived pointers and byte intrinsics; violations are counterexamples.

Slice ABI pairs (`%name.0` pointer plus the adjacent `i64 %name.1`) require
`--maxSliceLen K`. Length ranges over 0..=K; accesses and inbounds GEPs use
that exact symbolic length, never K. The symbolic byte array has capacity K. Whole-object zero-offset copies
reuse exact source arrays rather than constructing a store per byte.
The current argument model accepts noalias/noundef reference-shaped pointers
with a dereferenceable extent, or nonnull slice pairs; other pointer shapes,
byval/sret arguments and extents above 1 MiB remain unknown. LLVM opaque
pointers erase the source element type: the support claim is byte memory
under these artifact attributes, not reconstruction of arbitrary Rust types.
The fresh/disjoint input domain is explicitly recorded in the report.

Counterexamples include every initial byte up to the capacity, scalar lengths,
and byte-name mappings. Automatic native replay templates currently cover
scalar arguments only; pointer witnesses must be reconstructed explicitly.
For a harness runner that reads `//! phage-args:`, put `--maxSliceLen K` in
that line; the CLI itself takes the bound as an explicit option.

Calls into the crate and the standard library are executed. The first run
builds a cache of the sysroot's code under `~/.cache/phage/std/` (a few
seconds, once per toolchain). The property runs once, on one thread, in a
fresh program: statics start at their initializers and `getrandom` returns
arbitrary bytes, and values mixed from those bytes (hash keys) are treated as
uninterpreted functions whose counterexamples are confirmed exactly; the
report lists every such assumption a verdict used.
Other FFI, threads, async, `frem` and library math calls are unsupported
and give unknown. See
[CODEBASE_MAP.md](../CODEBASE_MAP.md) for the support table.

## Limits and evidence

```sh
target/release/phage check property.rs --function phage_target \
    --blockVisits 32 --maxStates 10000 --queryTimeout 15000
```

`--blockVisits` limits visits to each block within one call; every call
counts its own loop iterations. `--maxStates` limits total
explored states. `--queryTimeout` is per solver query, in milliseconds. Timeout diagnostics name
the query and limit. Cancelled incremental contexts are restarted; SIGINT
recovery requires a real push/check/pop probe before reuse.
`--maxMemoryMiB N` sets a sampled RSS budget for Phage and its child processes.
The default is min(8192 MiB, half of Linux MemAvailable), or 1024 MiB if
MemAvailable cannot be read. Samples run between instructions and during
solver waits, at most 5 ms apart when work continues. Exceeding the budget
returns unknown with `memoryLimit`; the selected limit is saved in RESULT.json.
This is a soft budget: a single instruction, compiler invocation or blocking
pipe operation can overshoot before the next sample. For hostile or stress
inputs, also apply an OS memory cap. Shared RSS pages may be counted twice.
A reachable
limit yields unknown; it is never assumed away. The earlier flags
`--unwind`, `--states` and `--timeout` are still accepted.

Each run prints a new directory in `results/`. It contains `RESULT.json`,
the LLVM artifact, compiler output, source snapshots and SHA-256 hashes,
versions, flags, limits, diagnostic reason codes, Rust source locations and
any concrete model. Solver checks, assertions and elapsed time are recorded;
`slowest-query.smt2` isolates the longest query for profiling. Counterexamples also get
`counterexample.smt2` and, when supported, `replay.rs`.

```sh
z3 results/RUN/counterexample.smt2
rustc --edition=2024 -C opt-level=1 -C overflow-checks=yes \
    results/RUN/replay.rs -o results/RUN/replay
results/RUN/replay
```

Replace `RUN` with the printed directory name. New templates reference captured
source snapshots, so relative module imports reproduce the recorded source.
Absolute imports or uncaptured build environment may still require the original
setup. Native
replay uses unwinding to catch panics; verification uses panic abort. The
verifier emits the template for review and does not execute it. The example
runner checks hashes and replays only our known local fixtures.

Object addresses are symbolic, so a counterexample may hold only for some
placements, such as a `u64` read from a byte buffer that is aligned on one
address and not on another. Such a model also lists the address it chose,
as a `placement:` line naming the object. A native replay places its objects
itself, so it may run cleanly; that does not refute the LLVM counterexample.

The retained LLVM artifact is the proof boundary. Results do not cover the
whole project, source behavior removed by optimization, or all Rust features.
This new interpreter needs further independent validation.

Pointer provenance survives unchanged ptrtoint/select/inttoptr choices, loads, stores and whole-pointer byte copies
at symbolic offsets, including String keys inside hash-map buckets. The
[String-map control](../examples/string-map.rs) proves lookup for two fixed-size
keys and every byte value; its deliberately wrong result is confirmed by
native replay. This does not establish all String-map operations. Heap sizes may be symbolic when their path proves a finite upper bound up to
1 MiB. Every access uses the exact requested length. Larger/unbounded symbolic
lengths and symbolic alignment remain unknown; no bound is assumed to gain a
proof. Layout/overflow errors still follow compiled Rust and allocator checks. The simplifier now uses deterministic
String-keyed `BTreeMap` caches: Phage proves a real miss/hit plus a bad-cache
mutant in 3–5 seconds. The complete `State::simplify` harness still exceeds
the 180-second runner limit, so it remains an opt-in inconclusive scalability
check. See [the retained evidence](../evidence/string-maps-work/SUMMARY.md).

## What works on Nexagate

- QUIC integer reader: all 8 byte inputs, lengths 0–8; checks truncation,
  decoded width and the 62-bit limit.
- Old HPACK decoder: finds the long-continuation shift panic; native Rust
  replay confirms it.
- Fixed HPACK decoder: proves the property for all 12 byte inputs,
  lengths 0–12 and prefixes 1–8.
- Overflow and unsupported-call examples exercise failure and unknown.
  Regression tests cover memory, poison and path limits.

These are the same two production functions checked earlier with Kani.
Kani has broader support and already proved seven Nexagate properties.
Shared-source repeated comparisons now show wins for QUIC, fixed HPACK and
finding the old HPACK defect. See [the measured comparison](../evidence/improvements/SUMMARY.md); this
does not establish that Phage is better overall.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
python3 tools/examples.py
python3 tools/issues.py
python3 tools/selfcheck.py   # Phage on its own source, with mutants
python3 tools/compare_kani.py --cases quic hpack --repeats 3
```

The runner keeps all logs and native/SMT replay results under `evidence/`.
See the [measured results](../evidence/first-engine/SUMMARY.md) and
[earlier attempts](../evidence/first-engine/ATTEMPTS.md).

## Lessons from Kani issues

[The issue review](kani-issues.md) tracks assertion forms, lost diagnostic
information, missing calls, solver crashes, const generics and profiling.
Tests distinguish passed properties, native-confirmed failures, unsupported
code and resource exhaustion. The current tool still requires an explicit
property; automatic generic harness generation is not implemented.

## For the next agent

Start with [AGENTS.md](../AGENTS.md) and [CODEBASE_MAP.md](../CODEBASE_MAP.md).
The production-gate research and roadmap is in
[docs/production-readiness-research.md](production-readiness-research.md).
The map lists modules, supported instructions, trust boundaries and extension
steps. Keep it current when behavior changes.

The round-2 controls prove the frozen IPC decoder for payloads up to 12 bytes
on both toolchains (with 200000 states), and bounded Vec push/slice copies and
String::from_utf8 for n <= 64. Off-by-one and unchecked-capacity controls fail
and are replayed natively. These are bounded artifact checks; they do not
verify all allocator sizes, every IPC payload, or the whole terminal.

Round-3 spec controls ensure fabricated integer pointers do not reserve memory
or separate real allocations, including allocations activated by lifetime.start.
Zero-length memcpy/memmove/memset still honor noundef alignment attributes.
A second lifetime.start resets a live object's bytes to uninitialized, as
[LangRef specifies](https://llvm.org/docs/LangRef.html#llvm-lifetime-start-intrinsic).
See evidence/round3-work/SUMMARY.md for commands and evidence boundaries.
