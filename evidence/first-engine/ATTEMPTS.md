# Earlier attempts retained

- Initial HPACK runs returned `unknown` for `llvm.experimental.noalias.scope.decl`, then for a global pointer operand. Alias metadata is now a no-op; global byte arrays and conditional pointers have explicit semantics. No unknown was called a proof.
- First native replay included both exported `ruhealth_target` harnesses in one binary. rustc rejected their duplicate symbol. Separate replay executables reproduced the old panic and checked the fixed code.
- First clippy run treated the verification harnesses in `examples/` as ordinary Cargo example binaries and rejected their missing main functions and export attributes. `autoexamples = false` now identifies these as verifier inputs; the engine still forbids unsafe code.
- A resource-exhaustion regression initially expected a returned verdict, but Z3 instead returned `push canceled`. The CLI already records solver errors as unknown. The control now accepts either unknown or an explicit solver resource error, and rejects a successful proof.
- Recorded initial timings before artifact collection: QUIC 0.859 seconds; old HPACK counterexample 20.364 seconds; fixed HPACK proof 4.264 seconds. Later runs with explicit pointer-address overflow checks took 64.881 and 13.148 seconds. These are different engine revisions, not interchangeable benchmark results.

- Review found that LLVM undef cannot be pinned to one symbolic value across uses. Observed undef now returns unknown; unused undef payloads remain allowed. A regression prevents proving `icmp eq` of a reused undef phi as a tautology. Earlier passing runs were superseded by a fresh run after this fix.

- Semantic review added immediate obligations for zero/poison divisors even if their result is unused. It also requires initialized, defined memory reads when the result is unused, conservatively covering load noundef metadata. New controls reject both cases. Earlier run directories remain retained.
