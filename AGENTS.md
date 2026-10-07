# Phage

Phage is an independent Rust verification engine. It uses rustc as the
Rust frontend and Z3 as a bit-vector/array solver; it does not call Kani or CBMC.

Read [CODEBASE_MAP.md](CODEBASE_MAP.md) first. Update the map when modules,
instruction support or trust boundaries change.

- Unsupported instructions, unresolved calls, solver unknown and reachable
  exploration limits are **unknown**, never a successful proof.
- Verify the compiled artifact and record rustc/Z3 versions, flags and input
  bounds. Do not present a bounded LLVM proof as verification of all Rust.
- Preserve original counterexamples and failed attempts. Compare against
  native Rust replay and Kani using the same production functions.
- Model integer widths, overflow flags, poison, stack bounds and lifetimes
  explicitly. Never treat undefined values as silently valid.
- Keep the implementation on Rust std. Z3 is invoked as a separate solver.
- Tests must include negative controls and unsupported-code cases; a tool
  that only prints success is not a verifier.
- Keep files below about 600 lines and document their responsibilities.
- Do not commit without the user's request.
