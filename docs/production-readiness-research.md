# Making `phage check` mean "ready for production"

Research of 2026-09-30. It combines measurements on this machine with a
review of the Rust-verification and formal-methods literature. Items marked
**measured** were run here, and the conditions are named. Citations were read
at the source unless marked *claimed*.

## 1. The honest target

No verifier can make "passes, therefore production-ready" true on its own.
A proof covers the properties that were stated, over the input bounds that
were given, for the compiled artifact that was checked, while trusting
rustc, LLVM, Z3 and Phage itself. Verified systems still ship bugs. They
come from wrong specifications and from unverified glue code, not from the
verified core:

- IronFleet, Verdi and Chapar: 16 bugs, of which 2 were in the spec and 11
  in the shim code ([Fonseca, EuroSys'17](https://www.cs.purdue.edu/homes/pfonseca/papers/eurosys2017-dsbugs.pdf)).
- Cedar: proofs found 4 bugs, while differential and property-based testing
  found 21 ([arXiv 2407.01688](https://arxiv.org/abs/2407.01688)).
- Firecracker, which runs Kani in CI, still had 2026 escape CVEs outside
  its verified components (*claimed*).

What we can build is a gate where every pass has a precise, checkable
meaning, where spec strength is measured, and where nothing silently falls
out of scope:

> `phage gate` passes = **every function in scope** reached its declared
> level, with **zero unknowns**, the declared **bounds and assumptions**,
> and a measured **mutation kill score** for its properties.

Section 5 lists the complementary evidence that only other techniques can
produce (concurrency, performance, configuration). The gate should collect
that evidence and report it, not claim to have proven it.

## 2. Where Phage stands (measured)

Benchmark: 29 independent cases, 18 real bugs and 11 fixed versions. Ground
truth came from native Rust: exhaustive where the domain is at most 2^26,
otherwise 3M edge-biased random samples. Suite and runners:
`~/.cache/phage-eval/` (`native.py`, `bench.py llvm,mir,kani`).

| Tool | Bugs caught | Correct code proved | Wrong verdicts |
| --- | --- | --- | --- |
| Phage LLVM, before 2026-09-30 evening | 11/18 | 7/11 | 0 |
| Phage LLVM, with intrinsics/switch/noreturn (now in tree) | **17/18** | 9/11 | 0 |
| Phage MIR engine | 9/18 | 4/11 | 0 |
| Kani 0.68.0 (CBMC 6.11) | 18/18 | 10/11 | 0 |

- Every Phage counterexample reproduced in native Rust. One case panics
  for a single key out of 2^32, which 3M random tests missed. Phage
  returned that exact key (`0x0b4f4092`) in 0.06 s.
- **Remaining misses:** heap `Vec` in the LLVM engine, and a nonlinear
  `checked_mul`-then-divide proof. Every solver timed out on the latter:
  Z3, Bitwuzla and cvc5 at 60 s each, Kani/CBMC at 300 s. cvc5's int-blasting mode
  proves the unsigned version in 0.03 s (section 7).
- **Speed:** on the 26 cases both tools solved, Phage's median full-CLI
  time was 0.11 s against Kani's 0.53 s. These are single runs at load
  averages of 1–18 on a shared 32-core host. Loop-heavy code is the
  exception: gcd took 10.8 s against Kani's 3.7 s (section 4).
- **Soundness probes:** a bug at loop iteration 41 under a 32-visit bound,
  u128 overflow, an unused overflow and unused out-of-bounds reads. No
  false proof resulted.
- **A bound caveat:** LLVM runtime-unrolled one loop by 8, so a 41-iteration
  bug needed about 6 block visits. `--blockVisits` bounds compiled blocks,
  not source iterations, and the report should say so.

### 2a. Held-out results after the 2026-09-30 night work (measured)

Protocol: each held-out set was written blind, its labels were confirmed
natively (exhaustive where the domain is at most 2^26, otherwise 3M
samples), and each tool ran it **once** with a frozen binary. Fixes made
afterwards are labeled post-hoc. Kani is 0.68.0 with `--unwind 33` and a
300 s limit per case. The machine is a shared 32-core host whose load
average stayed between 5 and 21, so times are single runs, not tight
measurements.

| Set (cases) | Phage one-shot | Wrong | Kani | Wrong | Median time, Phage / Kani |
| --- | --- | --- | --- | --- | --- |
| Held-out 3 (24) | 22 | 0 | 23 | 0 | 0.20 s / 0.92 s |
| Held-out 4 (24) | 19 | **1** | 20 | 0 | 0.10 s / 1.03 s |
| Held-out 5 (24) | 23 | 0 | 23 | 0 | 0.12 s / 1.73 s |
| HashMap/atomics controls (10, not blind) | 8 | 0 | 5 | 0 | — |

- The held-out 4 wrong answer was a false counterexample. Phage treated
  every read of uninitialized memory as UB. LLVM does so only for loads with
  `!noundef`, and BTreeMap copies `MaybeUninit` arrays with plain loads.
  The fix is a memory model that separates undef bytes (defined or not bit
  by bit) from poison bytes (any poison byte poisons the loaded integer).
  With it, the case now proves post-hoc.
- Kani's misses in these runs are all 300 s timeouts: BTreeMap/BTreeSet
  twice, HashMap and HashSet. It timed out on all five HashMap/HashSet
  controls. Phage proves or refutes most of them in 15–240 s and leaves
  the rest unknown after solver timeouts. It never gave a wrong verdict on
  them.
- Phage's held-out 5 miss (a rotate property) exposes an LLVM 21.1.8
  optimizer bug rather than a Phage gap. `opt -passes=bdce` removes
  `and i8 %k, 31` under `fshl` but keeps the call-site `range(i32 0, 32)`
  on the now-unmasked argument, which LangRef makes poison for k ≥ 32.
  Phage reports unknown with reason code `attributePoison`. Kani, which
  works from MIR, never sees that IR. The reproducer is
  `~/.cache/phage-eval/llvmbug/bdce-range.ll`. The same bug class was
  fixed in other passes ([llvm PR #224228](https://github.com/llvm/llvm-project/pull/224228),
  [#113997](https://github.com/llvm/llvm-project/issues/113997)); no BDCE
  report was found.
- **Soundness review loop.** Each semantic change was reviewed by codex in
  parallel read-only consults, one per subsystem. Every finding was checked
  in the code and its regression test was run against the old source. About
  17 confirmed false-proof bugs were found and fixed before any benchmark
  saw them. Examples: no-provenance pointers folded as null, a default
  atomic alignment, duplicated ODR globals, `nonnull`/`align`/`range` on
  arguments and results, `!noundef` spacing, vector ABI alignment, and
  oversized literals in the constant folder. One lesson: a differential test
  whose checker also runs the folder under test is circular.
- **Speed (single runs, loaded host).** Constant folding of ground terms
  (`src/ground.rs`, checked against Z3 on random terms) and literal branches
  halved the BTreeMap case, 207 s → 100 s, and cut a sort+dedup case from
  6.5 s to 3.6 s. Keeping never-poisoned and fully-initialized definedness
  arrays literal made HashMap ~17% faster. Binding long memory arrays to
  fresh names gave no gain and was reverted. HashMap/HashSet remain the
  slow class: SipHash with unknown keys feeds symbolic bucket indices.
  Treating the hash as an uninterpreted function is the next lever.

Current build (v8.1, all fixes, rerun on every set; held-out 3–5 are
post-hoc now): original 29/29, held-out 1 20/20, held-out 2 24/24,
held-out 3 24/24, held-out 4 23/24, held-out 5 23/24, controls 8/10, and 0
wrong verdicts anywhere. Summed times fell from v6 (held-out 4 469 s →
311 s, held-out 5 57 s → 36 s) while medians stayed near 0.1 s. The
remaining misses are two HashSet/HashMap solver timeouts and the LLVM-bug
case above. With `--queryTimeout 60000`, one of the timeouts (control c10)
turns into a found bug in 298 s.

New semantics since section 2 (all with negative controls): library
globals and thread-locals (fresh-program assumption, recorded), atomics
under one thread, `getrandom`, integer SIMD vectors, pointer provenance
through exact integer copies, per-byte definedness, argument and result
attribute contracts, null/poison dereference as a counterexample, and
Bitwuzla as an extra solver racer.

### 2b. Held-out 6 and the hash-table work (2026-10-01, measured)

Same protocol as 2a: written blind, labels confirmed natively (23 of 24
exhaustively), frozen binary v9 (`sha256 8de3b097…`, scratch commit
0728930, still named RuHealth then), each tool run once. Kani 0.68.0,
`--unwind 33`, 300 s per case. Host load average 5–12 during the runs.

| Held-out 6 (24 cases) | Correct | Wrong | Median time | Total time |
| --- | --- | --- | --- | --- |
| Phage v9, one-shot | **20** | 0 | 0.26 s | 356 s |
| Kani 0.68, one-shot | 17 | 0 | 3.6 s | 2225 s |

- Only Phage decided four cases: HashMap insert, BTreeMap range, BTreeSet
  first and an array rotate round trip. Kani hit its 300 s limit on all
  four. Only Kani decided one case, a float round trip; Phage has no floating
  point yet. Both missed three. Kani timed out on all of them. Phage hit
  an unmodeled `llvm.is.constant` and `invoke` inside std's `format!` path
  (two cases) and a 300 s timeout on a HashMap counted with
  `entry` + `values().sum()`.
- Post-hoc fixes for the two `format!` cases, all reviewed by codex:
  - `invoke` becomes a call plus a branch to its normal label. Std is built
    with unwinding, and a panic is already a reported failure.
  - `inttoptr` of an allocation's base plus an offset regains that
    allocation's provenance (exposed provenance; bounds are still checked).
  - A surviving `llvm.is.constant` is false, as code generation lowers it.
  With them, held-out 6 is 22/24 (`phage-v9.1`, `sha256 6a226662…`).
- Rerun of every earlier set with that build: original 29/29,
  held-out 1 20/20, 2 24/24, 3 24/24, 4 24/24, 5 24/24, controls 10/10.
  There are 0 wrong verdicts anywhere. Held-out 3–5 and the controls have
  been seen before, so they are regression checks, not blind evidence.

What made hash tables fast and decidable. Each item is listed with what it
fixed, as single runs on the loaded host:

1. **Hash-consed `define-fun` names.** One solver name per distinct term,
   so equal computations are one term.
2. **Store-to-load forwarding.** A load of exactly a stored whole value
   returns that term, with identical per-byte definedness. Every byte
   writer invalidates it.
3. **Canonical XOR terms.** Commutative operands are ordered, and XOR /
   `or disjoint` chains become a sorted leaf set plus one constant. The
   SipHash copies inlined into `insert` and `remove` were equal but written
   differently (`(k ^ x) ^ C1` vs `(k ^ (x | B)) ^ C2`). Now they are one
   term, and HashMap insert→remove proves (control c09: unknown → 10 s).
4. **Shaped opaque functions.** A whole opaque computation over hash-key
   bytes is one uninterpreted function of its leaves, keyed by its
   interned shape. This replaces one function per SipHash round, whose
   hundreds of applications made the solver's congruence (Ackermann)
   reasoning quadratic. A HashSet bug (held-out 4 m23) went from unknown
   to found in ~25 s. Candidates are confirmed with exact semantics, by a
   concrete witness first.
5. **Known-bits and bounds simplifier** (`src/knownbits.rs`). LLVM's
   KnownBits transfer functions plus unsigned bounds rewrite every term
   before it is named. They discharge the always-true `nuw` / `nsw` /
   `disjoint` / in-bounds conditions and resolve `select` over store chains
   for every index in a range. Allocation-base alignment counts as a known
   fact. The work roughly halved hash cases.
6. **SIGINT instead of solver restarts.** When a racer won, the incremental
   Z3 used to be killed and all definitions replayed. That made identical
   runs vary 3–6×: BTreeMap m01 took 90–600 s. Now the check is cancelled in
   place, and m01 takes 51 s in every run.

Control times, v8.1 → v9.1:

| Control | v8.1 | v9.1 |
| --- | --- | --- |
| c01 HashMap overwrite (ok) | 93 s | 14 s |
| c02 HashMap overwrite (bug) | 92 s | 13 s |
| c03 HashMap two keys (bug) | 247 s | 25 s |
| c09 HashMap insert→remove (ok) | unknown | 10 s, proved |
| c10 HashSet (bug) | unknown | 25 s, found |

Kani timed out on all five.

**Soundness review.** Codex reviewed every change in parallel read-only
consults, then re-checked each fix. Each finding was confirmed in the code,
and each regression test fails on the old source. The findings:

- **Opaque abstraction lost per-byte definedness:** a false counterexample.
- **A shift-identity rewrite with an overflowing amount:** a false
  rewrite. A shift amount of 2^32−1 wrapped to "true".
- **Undef read as poison:** a false counterexample (`and undef, 0`).
- **The first fix for that (a fresh symbol per observation):** this was
  unsound for derived values. The final design treats an observed undef as
  undefined bytes, exactly like uninitialized memory: masks recover exact
  bytes, and branching on undef is UB.
- **A per-byte value commuted to the right of a constant mask:** a false
  counterexample.
- **`llvm.is.constant` modeled as nondeterministic:** a false
  counterexample.
- **Atomic exchange dropped per-byte definedness:** a false counterexample.

Open after this round:

- HashMap probing with three symbolic inserts. Its hardest query is a
  pigeonhole argument over the control-byte group: Z3 standalone takes 10 s
  and the in-tool query exceeds its budget.
- Floating point.
- An undef select condition with different arms is treated as poison:
  strict, so it can produce a false counterexample in exotic code.

### 2c. 24/24 and floating point (2026-10-01, measured)

Build v10 (`sha256 625c1b7a…`). These are post-hoc results: held-out 6 was
already seen in 2b. Every set is now complete, and there are 0 wrong
verdicts:

| Set (cases) | Correct | Total time, v9.1 → v10 |
| --- | --- | --- |
| Original (29) | 29/29 | 18 s → 13 s |
| Held-out 1 (20) | 20/20 | 14 s → 11 s |
| Held-out 2 (24) | 24/24 | 11 s → 4 s |
| Held-out 3 (24) | 24/24 | 18 s → 5 s |
| Held-out 4 (24) | 24/24 | 115 s → 12 s |
| Held-out 5 (24) | 24/24 | 25 s → 7 s |
| Held-out 6 (24) | **24/24** | 382 s → 125 s |
| Controls (10) | 10/10 | 88 s → 16 s |

Selected cases, v9.1 → v10 (single runs, load average about 6):

| Case | v9.1 | v10 |
| --- | --- | --- |
| BTreeMap m01 | 66 s | 1.8 s |
| BTreeSet n12 | 18.5 s | 0.8 s |
| HashMap k16 | 12.9 s | 1.8 s |
| HashMap counting p01 | timeout | 106 s, proved |
| HashMap/HashSet controls | 10–25 s | 1.6–3.9 s |

Kani needs a 300 s timeout on all of these.

- **Floating point** (`src/floats.rs`). A value keeps its IEEE bits, so
  bitcast, memory, select and the sign operations are exact, NaN payloads
  included. Arithmetic, comparisons and conversions go through SMT-LIB's
  FloatingPoint theory with round-to-nearest-even. A computed float gets
  fresh bits whose FP reading is the result, so a computed NaN is quiet but
  may carry any payload. Fourteen float properties were checked against
  native Rust over all 256 inputs. Six natively failing ones (0.1 × 10,
  sqrt², NaN self-compare, signed zero, clamp bounds, 1/0) are found as
  counterexamples. The eight that hold natively are proved.
- **Allocation-fact slicing** (`src/solver/slice.rs`). Every allocation adds
  facts about its base address: nonzero, in range, aligned, disjoint from
  every live object. A query now keeps only the facts whose bases it
  mentions. These facts were 323 of 332 assertions in a typical HashMap
  query, and dropping them is most of the speedup above.
  - Codex refuted the first exactness argument with an alignment-2^63
    object. Slicing is now exact only under explicit bounds: at most 2^32
    bytes per object, alignment at most 2^12, at most 2^20 objects
    including literal-address ones. Past them it switches off.
- **Self-tuning race stage.** The wait before racing other solvers halves
  whenever a racer wins and grows back whenever the incremental solver
  wins. On p01, Bitwuzla decided 536 of 880 queries; this change took p01
  from 170 s to 106 s.
- **Signed comparisons** are decided from known bits and bounds. Without
  this, the control-byte masks of SwissTable stayed symbolic.
- **Tried and removed: splitting hash bucket indices into concrete cases.**
  It made the two-key cases about 3× faster. With three keys it multiplied
  paths 8× per key: p01 went to the state limit with full splitting and
  timed out with two splits per path.
- **The LLVM BDCE bug from 2a is already fixed upstream.** On Compiler
  Explorer the reproducer still fails on `opt` 20.1, 21.1 and 22.1, and is
  fixed in 23.1 and trunk. The fix is
  [llvm/llvm-project#195516](https://github.com/llvm/llvm-project/pull/195516),
  "[IR] Drop parameter attributes", merged 2026-05-03. No new report was
  filed. Rust 1.94 uses LLVM 21, so rustc picks up the fix with LLVM 23.

Codex findings in this round, all fixed with regression tests:

- the slicing placement argument (unbounded sizes and alignments,
  literal-address obstacles);
- signaling-NaN results;
- `minnum`/`maxnum` making one choice for opposite zeros across calls;
- fast-math flags on calls being ignored;
- implicit `alloca` alignment.

Open:

- HashMap with three symbolic keys still takes about 100 s. Its queries are
  genuinely hard: Bitwuzla needs about 0.5 s each and Z3 about 13 s.
- `frem`, libm calls, half/fp128 and float vectors are unknown.

### 2d. Running Phage on repomap's readers (2026-10-01, measured)

Property: repomap's C, Go and TS/JS readers never panic on short symbolic
inputs (harnesses and logs in `~/.cache/repomap-phage`). Phage v10 re-found
the multibyte slicing panic planted back into the C reader in 0.8 s and
proved two small pieces, but four Phage gaps blocked the rest:

1. **Aligned word reads after `align_offset`** (`memchr` on 16+ byte
   haystacks, so `contains(char)`, `lines()`, `split`): accesses were
   refused when their alignment exceeded the object's declared one. Now the
   address itself must be aligned (LangRef); see the README placement note.
2. **Literal addresses in phi incomings** (`NonNull::dangling()` as
   `inttoptr (i64 1 to ptr)`): phis read the predecessor state, which the
   per-instruction prescan never prepared. Globals named only by a phi had
   the same gap (found by codex review). Both are created at block entry.
3. **A select whose arm is a select over undef**: the result is undef where
   the chosen arm is, else the chosen arm's value; a plain undef condition
   over structurally equal arms of any kind returns the arm (codex review:
   the old integer-only shortcut gave a false counterexample there).
4. **Block visits counted per path across calls**: four `rfind` calls with
   a 13-byte needle shared one 32-visit budget in `StrSearcher::new`. Each
   call frame now counts its own visits, as the MIR engine already did; the
   caller's counts are restored at return, so a caller loop stays bounded.

After the fixes the three harnesses that had stopped on gaps 2-4 run into
the state limit instead. With `--maxStates 2000000` the typedef harness
(4 symbolic bytes) found a real repomap panic in 497 s: input
`28 00 D0 80` ("(\0Ѐ") reaches `&group[1..close]` in `typedef_name` with an
unbalanced `(`, where `close` lands inside the two-byte character. Native
replay confirms it ("byte index 3 is not a char boundary").

The eval kit is unchanged by all four fixes: 29/29, 20/20, 24/24 on
held-out 2-6, controls 10/10, no wrong verdict and no verdict changed.

### 2e. Phage on Phage (2026-10-01, measured)

`tools/selfcheck.py` generates harnesses from the current `src/` (std-only
modules copied whole, other functions extracted verbatim, private items
reached through an appended child module) and requires each harness to be
proved and each deliberate mutant to give a counterexample.

- **Known-bits transfer functions** (and, or, xor, add, mul, not, sub, neg,
  shl, lshr, ashr with known bits and bounds): proved for every 8-bit input
  in 2-4 s; six unsound mutants all give counterexamples. These functions
  rewrite every solver query, so this is a soundness result for Phage.
- **`heap::literal`**: proved over every 5-character string from its own
  alphabet in 16 s; the decimal-for-hex mutant is caught.
- **Path explosion:** the parsers over arbitrary bytes (4-6 symbolic bytes)
  hit 10,000-200,000 states or 15-40 minute limits. Phage explores path by
  path without merging states, so each byte multiplies the paths by its
  character classes; Kani's single formula avoids that. Harnesses therefore
  draw characters from the parser's alphabet. Even so, 5 characters from an
  8-letter alphabet gave no verdict within 40 minutes for `ir::split` and
  `ir::typed`, the S-expression parser and `layout::check`.
- **Found and fixed:** a closed stdout (`phage check x | head`) made
  `println!` panic, turning the verdict's exit code into 101 (now ignored by
  the `say!` macro; the report is on disk first). `select %c, undef,
  <undef phi>` under a condition that may be poison (in
  `u128::from_str_radix`) was unsupported; it is now undef where the
  condition is defined and poison elsewhere, for whole-byte widths only
  (codex: below a byte an observed undef counts as poison, which would give
  a false counterexample).
- **Symbolic pointer gap repaired:** pointer shadow slots now carry exact
  offsets and presence guards. String pointers copied into symbolic
  hashbrown buckets survive; loads recover only a matching active slot whose
  bytes are proved intact. Whole-pointer copies, overwrite guards, poison,
  lifetimes and realloc prefixes have positive, negative and unknown controls.
  The same-source `HashMap<String, u32>` lookup harness proves for two
  five-byte keys and every byte value, and its wrong-result control produces
  a native-confirmed counterexample. Kani's shared positive harness timed out
  at 180 s (retained first attempt); this is no general speed ranking.
- **Cache self-check improved:** restricted constant GEP operands now retain
  their global provenance and bounds. The simplifier's four String-keyed
  caches use deterministic `BTreeMap`s, removing random-hash exploration from
  Phage-on-Phage. A real known-bits cache miss followed by a hit proves in
  3.1 s; returning a deliberately wrong cached value gives a counterexample
  in 4.1 s. This cache property is now part of the default self-check gate.
  Exact pointer-undef comparison followed by `or true` or `and false` is also
  modeled; poison remains a counterexample.
- **Remaining full-path gap:** `State::simplify` twice over one fixed identity
  still exceeds the 180 s runner limit in both original and mutant. This is
  retained as `tools/selfcheck.py --only cache-full`; both outcomes are
  inconclusive, never proof or counterexample. The established
  transfer/cache/literal controls remain the default gate. Reports,
  source snapshots, native replay and failed attempts are indexed in
  [String-map evidence](../evidence/string-maps-work/SUMMARY.md).

## 3. Proposed gate levels

The levels are adapted from SPARK's Stone–Platinum ladder
([SPARK UG](https://docs.adacore.com/spark2014-docs/html/ug/en/usage_scenarios.html)),
where Silver means absence of runtime errors and is the industry default.

| Level | Meaning | User supplies |
| --- | --- | --- |
| L0 Inventory | Every function in scope is listed as modeled, unknown (with reason code) or out of model (unsafe, FFI, heap, async, threads, uninlined dependency calls). | Scope |
| L1 Crash-free | An automatic harness per function, all arguments symbolic within bounds. No panic, overflow, poison, out-of-bounds or invalid memory. | Preconditions, bounds |
| L2 Properties | User properties hold. They count only after passing the strength checks in section 6. | Properties |
| L3 Reference equivalence | The implementation equals a small executable model on all bounded inputs. The same model drives differential fuzzing beyond the bounds, following the ShardStore and Cedar pattern. | Reference model |

Rules:

- A level is claimed per function. A crate reaches a level only when 100%
  of its scope does.
- `unknown` never counts toward a level. Counterexamples break the build.
- The unknown count is a ratchet that may never increase. Google found that
  build-breaking checks must have no "effective false positives"
  ([CACM'18](https://cacm.acm.org/research/lessons-from-building-static-analysis-tools-at-google/)).
- The report has the shape of SPARK's `gnatprove.out` and Kani's
  autoharness table: verified, unknown by reason, counterexamples, bounds,
  exclusions and the trusted base (rustc/LLVM flags, Z3 version, Phage
  commit).

**Profile gap.** Proofs use `opt-level=1`, `overflow-checks=yes` and
`panic=abort` (`src/main.rs`). Release builds normally wrap on overflow.
Excluded inputs are therefore unchecked in release, and the report must say
this. Better still, the gate can add a pass with release flags that treats
wrapping as allowed only where the source asks for it.

## 4. Capability roadmap, ranked by impact on the goal

1. **Whole-program MIR, including std and dependencies.** This replaces
   hand-written std models, which will never be finished. It is the root
   cause of the MIR engine's 16 unknowns (`Range`, slices, `min`, generics,
   `Drop`). Recipe, verified locally by probe:
   - Build a MIR sysroot once per rustc commit, in about 19 s:
     `RUSTFLAGS="-Zalways-encode-mir -Zmir-opt-level=0 -Zmir-preserve-ub -Cpanic=abort" cargo build -Zbuild-std=core,alloc,std,panic_abort`.
     Keep std's own overflow-check and debug-assert settings, and not
     `cfg(miri)`.
   - Add a small sidecar exporter on `rustc_public` that walks monomorphized
     instances from the entry points, using Kani's `reachability.rs` as the
     template. It writes `program.mirx.json`, which Phage reads with a
     std-only JSON parser.
   - The core stays dependency-free. The exporter runs as a separate
     process, like Z3.
   - Charon (`--preset soteria`) is the second, independent frontend for
     cross-checking.

   Stable MIR has no stable serialized form yet
   ([goal #266](https://github.com/rust-lang/goals/issues/266)), so the
   exporter owns a versioned schema and pins nightly.
2. **`cargo phage` plus autoharness.** This reaches L1 for every public
   function without hand-written properties. Kani's autoharness generated
   16,748 harnesses for std and verified 11,970. 56% of the skips were
   generics and 22% missing `Arbitrary` implementations
   ([std paper, NFM 2026](https://arxiv.org/abs/2606.17374)). So support
   explicit generic instantiation and derive symbolic values for user
   structs and enums from MIR. Skipped means unknown, listed with a reason.
3. **Function contracts.** Adopt Kani's `requires`/`ensures`/`modifies`
   attribute syntax so annotations work in both tools. Prove each contract
   once, then use it as a summary at every call site. This removes "general
   call → unknown" and is the biggest scaling lever: contract reuse cut a
   Firecracker proof from hours to seconds
   ([Kani paper, 2026](https://arxiv.org/abs/2607.01504)).
4. **Heap and aggregates.** Add a symbolic heap for `Vec`, `Box` and
   `String`, the benchmark's last LLVM miss, or get it for free through
   item 1 by executing alloc's own MIR over a byte-precise allocation
   model.
5. **Unbounded loops.** Try k-induction first, since it needs no
   annotations. Then add `loop_invariant`/`loop_modifies`/`loop_decreases`,
   checked by induction as in Kani. This turns block-limit unknowns into
   proofs.
6. **Rust-level UB outside the LLVM model.** Aliasing and validity checks
   are what Miri covers (Stacked/Tree Borrows). The gate should run every
   generated `replay.rs` and counterexample test under Miri. The LLVM path
   cannot see UB that rustc has already optimized away.
7. **Soundness hygiene as a feature.** Kani 0.68 reported SUCCESS when
   CBMC ran out of memory ([#4905](https://github.com/model-checking/kani/issues/4905),
   closed 2026-09-29). Phage's rule that unknown never passes, its
   solver-exit checks and its negative controls are real differentiators.
   Keep adding one per bug class.

## 5. What a verifier structurally cannot certify

`phage gate` should collect evidence for each of these rather than claim
them:

| Risk | Complement |
| --- | --- |
| Wrong or weak specification | Mutation kill score, reference models, review (section 6) |
| Inputs beyond the bounds | The same `bool` property run as a native fuzz or proptest target |
| Concurrency, async, atomics | loom for small primitives, Shuttle end to end (ShardStore's split) |
| Performance, latency, algorithmic DoS | Noise-aware benchmarks; PerfFuzz/SlowFuzz |
| Stack overflow, OOM | Stack analysis, fuzzing under memory limits (even SPARK Silver excludes these) |
| unsafe, FFI, aliasing | Miri, sanitizers, a cargo-geiger inventory |
| Configuration and environment | Tests with production-like and minimal configs |
| Liveness and protocol design | TLA+ or other design-level model checking |
| The trusted base (rustc, LLVM, Z3, Phage) | Negative controls, native replay, cross-checks against Kani and Charon |

## 6. Measuring spec strength automatically (self-improving)

A proof of a weak property is false confidence. Five mechanisms make
strength visible, and they tighten over time:

1. **Mutation kill score.** Mutate the code under test, using cargo-mutants
   operators and operator swaps, and rerun `check`. A `proved` mutant is a
   spec gap, reported with its line.
   - Mutants whose optimized IR hash equals the original's are equivalent
     and can be skipped for free; this pruned about 28% in
     [ICSE'15](https://discovery.ucl.ac.uk/1499169/1/Jia_Trivial_Compiler_mutation-testing-papadakis-icse15.pdf).
   - Evidence that it catches real weaknesses: a CBMC quicksort harness
     left 71 of 81 mutants alive at bound 1 but 8 at bound 3
     ([Groce, ASE'18](https://agroce.github.io/asej18.pdf)).
   - Used the same way here: all 14 non-equivalent mutants of the new
     intrinsic semantics were killed by the tests.
2. **Bounds by evidence.** Raise bounds until the kill score stops rising,
   and report whether the chosen bound is stable.
3. **Vacuity and exclusion audit.**
   - Prove the code under test is reachable with a non-excluded input.
   - Replacing the property with `false` must yield a counterexample.
   - Removing each `return true` exclusion in turn shows whether it hides a
     failure or is redundant.
4. **Proof coverage.** Report the source lines reached by completed paths,
   like Kani's `--coverage`.
5. **Escaped-defect ledger.** Every bug later found by fuzzing, replay or
   in production becomes a mutation operator and a regression property.
   Surviving mutants may only decrease. If fuzzing ever finds a failure in
   a function Phage marked `proved`, that is a verifier soundness bug.
   It becomes a permanent negative control.

## 7. Speed plan

**Measured:** Phage's time is almost entirely solver time, and mostly the
number of queries rather than hard ones. gcd spends 10.95 of 11.01 s in Z3
over 147 checks. The HPACK proof makes 486 checks, the slowest taking
0.47 s. Z3 also runs slower inside Phage's incremental push/pop session
than as a fresh process: one gcd query took 1.23 s against 0.33 s. Z3's
developers confirm that incremental mode drops its bit-vector tactic
pipeline ([Z3 #5046](https://github.com/Z3Prover/z3/discussions/5046)).

Levers, ranked by expected speedup against cost:

1. **Query layer (KLEE-style).** Send only the constraints connected to an
   obligation by shared symbols. Keep an unsat-subset cache, which is sound
   because any superset of an unsat set is unsat. Answer "sat" from a
   cached model only after evaluating that model exactly. KLEE measured
   15x with both techniques together (13,717 → 699 queries, 300 → 20 s,
   [OSDI'08](https://llvm.org/pubs/2008-12-OSDI-KLEE.pdf)). This fits
   Phage's many-cheap-queries profile.
2. **Persistent verdict cache plus per-function parallelism.**
   - Key the cache on the LLVM or MIR body hash, its callees, the rustc and
     solver versions, flags and bounds. Never cache unknown.
   - Unchanged code then costs 0 s in CI. Dafny/Boogie caching gave more
     than 10x in real sessions
     ([Leino and Wüstholz](https://pm.inf.ethz.ch/publications/LeinoWuestholz2015.pdf)).
   - Run one solver per worker; the host has 32 cores.
3. **Escalating solver portfolio.**
   - After about 200 ms, re-issue the query to a fresh Z3, then race
     Bitwuzla against cvc5 with `--solve-bv-as-int=sum`. If they disagree,
     return unknown and keep the query.
   - **Measured on Phage's own emitted query** (32-bit `checked_mul`
     then divide back): cvc5 with int-blasting returned unsat in 0.03 s.
     Z3, Bitwuzla and default cvc5 all timed out, and so did Kani/CBMC at
     300 s. This turns the one benchmark case every tool failed into a
     proof.
   - Bitwuzla on its own gave identical verdicts on all 29 cases and about
     2x on solver-heavy ones (gcd 10.8 → 5.4 s), even through a Python
     bridge.
   - SMT-COMP 2025, incremental QF_Equality_Bitvec (the division that holds
     QF_AUFBV): Bitwuzla 4,904 solved, Yices2 4,855, cvc5 4,255. Z3 did not
     enter ([results](https://smt-comp.github.io/2025/results/qf_equality_bitvec-incremental/)).
   - Int-blasting still fails on the *signed* variant; that remains open.
4. **Path merging at post-dominators.** CBMC and Crux build one formula per
   loop exit. Phage forks a path per iteration, which is the gcd gap
   (10.8 s against Kani's 3.7 s). State merging with query-count estimation
   gave "several orders of magnitude" on Coreutils
   ([PLDI'12](https://www.plai.ifi.lmu.de/publications/pldi12-statemerging.pdf)).
   Start with acyclic diamonds inside loop bodies.
5. **k-induction, then loop invariants and CHC as a fallback.** These turn
   bound-limited unknowns into proofs, and are faster than deep unrolling.
   - Kani's gcd with an invariant passed in 0.54 s, while bounded u64 gcd
     did not finish in an hour
     ([Kani paper](https://arxiv.org/abs/2607.01504)).
   - On 6,024 C tasks, k-induction produced 2,158 proofs against 1,211 for
     BMC ([CPAchecker](https://arxiv.org/pdf/2208.05046)).
   - Use Eldarica (first in the CHC-COMP 2025 bit-vector track) as the CHC
     fallback, and re-check every returned invariant in Z3.
6. **Contracts as call summaries.** See section 4, item 3.

**Negative result:** Z3's `check-sat-using qfbv` inside the session was
slower (gcd 21 s against 16 s at load ~30). Use a fresh process or another
solver instead of tuning tactics.

## 8. Suggested order

1. Done 2026-09-30: integer intrinsics, `switch`, `llvm.assume` as an
   obligation, `noreturn` core/alloc failure paths and quote-aware call
   parsing, reviewed by codex. Benchmark: 11 → 17 of 18 bugs.
2. The gate report format: L0 inventory, the unknown ratchet, the profile
   gap and the trusted base. This is cheap, and it makes every later step
   honest.
3. The query layer (slicing plus an unsat cache), the verdict cache, per-function
   parallelism, and the escalating Z3 → Bitwuzla / cvc5 int-blasting portfolio.
4. The MIR sysroot plus the `rustc_public` exporter (item 1), with
   `cargo phage` and autoharness on top.
5. Contracts, then the heap, then loop invariants and k-induction.
6. Mutation score, vacuity audit and the escaped-defect ledger, wired into
   `phage gate`.
