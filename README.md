# Phage

### Find the input your tests missed.

**Check Rust properties across symbolic inputs.** Phage returns a bounded proof, a counterexample you can investigate, or an explicit `unknown`.

An independent Rust verification engine: **rustc compiles, Phage explores, solvers decide**. Zero Rust crate dependencies. Checks include panics, overflow, invalid memory access and user-written properties.

## Speed, with receipts

Three shared-source parser checks against **Kani 0.68.0**:

| Check | Phage¹ | Kani | Speedup |
| --- | ---: | ---: | ---: |
| QUIC integer reader | 0.285 s | 8.895 s | **31.2×** |
| Fixed HPACK decoder | 1.561 s | 6.078 s | **3.9×** |
| Find the old HPACK defect | 2.326 s | 8.268 s | **3.6×** |

¹ Recorded as RuHealth, Phage's former name, on 2026-09-30. Medians of three alternating full-CLI runs after warmup, both tools pinned to CPU 0 on a loaded Linux x86_64 host; bound 32, identical properties. Phage used rustc 1.94.1 / Z3 4.16.0. These are parser-workload wins, not a universal speed ranking. [Commands, load, hashes and raw results →](evidence/improvements/SUMMARY.md)

## Why use it?

- **Search beyond examples:** explore the input domain of your property within explicit limits.
- **Keep the evidence:** reports retain source/artifact hashes, assumptions, compiler versions and replay material.
- **Fail honestly:** unsupported code, timeouts and reachable limits stay `unknown`.

## Try it

Requirements: Rust, Z3 and matching LLVM tools on Linux x86_64.

```sh
cargo build --release
target/release/phage check examples/safe-add.rs
```

**Only `proved` passes a proof-required gate.** A proof covers the retained artifact, property, bounds and model assumptions—not all Rust or your entire application.

[Write your first property →](docs/guide.md#write-a-property) · [Full guide →](docs/guide.md) · [Supported features →](CODEBASE_MAP.md) · [Research and limitations →](docs/production-readiness-research.md)
