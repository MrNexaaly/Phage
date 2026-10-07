# String-keyed map memory support (2026-10-01)

The unsupported symbolic pointer load/copy gap is repaired. This is a
bounded compiled-LLVM result, not verification of all String maps or Phage.
The production cache miss/hit is now proved with a mutation control. The
complete `State::simplify` path remains inconclusive at the runner limit.

## Change

Pointer shadows now have exact object-relative offsets and presence guards.
Whole pointers survive stores and memcpy/memmove at symbolic offsets, with
source-containment guards when the length is symbolic. Overwrites and fills
invalidate old slots under overlap guards. Realloc keeps only whole slots in
the retained prefix; a new lifetime clears slots. Reads split feasible slot
choices and re-prove the address bytes before recovering provenance. Missing
provenance, partial/corrupt slots and solver uncertainty never become proofs.
No allocator layout, access alignment, poison, bounds or lifetime checks were
removed. Core implementation remains Rust std with external solvers.

## Bounded shared property

[string-map.rs](../../examples/string-map.rs) calls the actual
`HashMap<String, u32>` implementation. `key: u8` chooses `alpha` or `bravo`;
`value: u8` is stored and read back through a borrowed string key. The negative
entry compares against `value + 1`. Both tools use these same function bodies.

| Tool | Positive property | Deliberate bad result |
| --- | --- | --- |
| Retained pre-change Phage | unknown: symbolic pointer load | unknown: symbolic pointer load |
| Updated Phage | proved | counterexample |
| Kani 0.68.0 / CBMC 6.11.0 | runner timeout, inconclusive | runner timeout, inconclusive |

[Shared comparison](../comparison-1790875911444622405/RESULT.json) retains
one warmup and one measured attempt per tool/property, command lines, versions
and 72 source/binary hashes. Kani's limit was 60 seconds per attempt; the
[earlier positive attempt](../comparison-1790874530951039463/RESULT.json)
also timed out at 180 seconds. No general performance ranking is claimed.
The first comparison was interrupted during its negative warmup after its
timeout runner leaked solver descendants; the repaired runner kills the whole
process group. A separate live cleanup control confirmed its child stopped.

Final [positive report](../../results/1790879843771718208-3/RESULT.json)
and [negative report](../../results/1790879843770912894-3/RESULT.json)
retain the actual LLVM, captured source, compiler flags, solver assumptions,
versions and bounds. Rustc was 1.94.1 / LLVM 21.1.8 and Z3 was 4.16.0.
Overflow checks were enabled, optimization was 1, panic was abort, and the
bounds were 32 block visits, 32 call frames, 10,000 states and 15,000 ms per
query. Heap allocation success and the existing single-thread/random-source
models remain explicit assumptions.

Native replay from captured source confirms `key=0, value=255` returns false
for the negative entry; the saved SMT query replays as sat. Separate native
execution checked all 65,536 input pairs for each property: every positive
result was true and every deliberate bad result was false. Native execution
uses its own allocation placements and random seeds.

## Controls and remaining gaps

- 147 Rust tests passed, including pointer-undef absorption and constant-GEP
  controls in addition to eight symbolic provenance tests with positive,
  negative, unsupported-provenance and reachable-limit cases. New coverage includes
  symbolic reads/stores, i64 pointer copies, symbolic copy lengths, partial
  copies, fills, reallocations, poison, lifetime and bounds failures.
- Formatting, Clippy with warnings denied, release build, all 11 end-to-end
  examples, seven issue controls and eleven default self-check cases passed.
- [Shared production regression comparison](../comparison-1790876609820942233/RESULT.json):
  Phage and Kani both proved the QUIC and fixed HPACK properties in all three
  measured trials, using the same existing production-function harnesses.
- The simplifier's deterministic String-keyed cache miss/hit proved in 3.1 s;
  its wrong-value mutant gave a counterexample in 4.1 s. `cache` now joins the
  default transfer/literal gate. The change did not materially alter the
  three-run QUIC/HPACK medians: 0.133 to 0.127 s and 0.299 to 0.306 s
  respectively ([comparison](../comparison-1790879549133358593/RESULT.json)).
- The complete fixed-input `State::simplify` original and identity mutant both
  exceeded 180 seconds. `tools/selfcheck.py --only cache-full --timeout 180`
  preserves that inconclusive boundary. Variable allocation sizes also remain
  unsupported (the retained first String-map attempt used unequal key lengths).
  Attempts have unique configurable directories and timeouts stop descendants.

The [manifest](RESULT.json), logs in this directory, comparison directories,
and self-check directories preserve all attempts. Earlier HashMap-cache
timeouts remain under `/home/zero/.cache/phage/selfcheck/1790874467430517523`,
`1790874705258185920`, and `1790875911430162769`. The current full-path
timeouts are retained under `selfcheck-work-btree/1790878681210852484` and
`1790879063123349154`; the successful default gate is under
`selfcheck-final-current/1790879914634829115`. The constant-GEP run interrupted
after passing its former failure point remains `results/1790878175297624868-2`.

Existing dirty work was preserved. No commit was made.
