#!/usr/bin/env python3
"""Phage on Phage: verify parts of Phage's own source with Phage.

Each harness is generated from the CURRENT src/ (whole std-only modules are
copied; other functions are extracted verbatim), so the checks follow the
code without edits. Every harness must be proved, and every mutant (one
deliberately wrong edit of the copied code) must give a counterexample: a
proof whose mutants also pass checks nothing. A mutant whose text no longer
occurs in the source fails loudly instead of silently testing nothing.

Usage: python3 tools/selfcheck.py [--phage PATH] [--only NAME ...] [--timeout SECONDS]
The default checks transfer functions, a production String-keyed cache lookup
and the literal parser, including a deliberate mutant for each property.
`--only cache-full` exercises all of `State::simplify`; it is retained as an
experimental scalability check until both the original and mutant finish.
Work files go to unique directories in ~/.cache/phage/selfcheck/.
"""

import argparse
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SRC = REPO / "src"
WORK = Path.home() / ".cache" / "phage" / "selfcheck"


def extract(text: str, signature: str) -> str:
    """One item by its signature prefix, matching braces outside strings,
    char literals and comments."""
    start = text.index(signature)
    i, depth = text.index("{", start), 0
    while i < len(text):
        c = text[i]
        if text.startswith("//", i):
            i = text.index("\n", i)
            continue
        if c == '"':
            i += 1
            while text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^'\\])'", text[i:])
            if m:
                i += len(m.group(0))
                continue
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
        i += 1
    raise ValueError(f"unbalanced item {signature}")


def shims() -> str:
    """The two helpers knownbits.rs reaches outside itself, verbatim."""
    literal = extract((SRC / "heap.rs").read_text(), "pub(crate) fn literal(expr: &str)")
    bv = extract((SRC / "value.rs").read_text(), "pub fn bv(value: u128, width: u32)")
    return f"mod heap {{\n{literal}\n}}\nmod value {{\n{bv}\n}}\n"


TRANSFER_CHECK = """
/// Phage self-check (tools/selfcheck.py): transfer functions are sound for
/// every 8-bit input, bounds included.
pub mod selfcheck {
    use super::*;
    fn contains(k: Known, v: u128) -> bool {
        v & k.zero == 0 && v & k.one == k.one && k.lo <= v && v <= k.hi && v <= mask(k.width)
    }
    fn input(zero: u8, one: u8, lo: u8, hi: u8, v: u8) -> Option<Known> {
        if zero & one != 0 {
            return None;
        }
        let k = Known::bits(u128::from(zero), u128::from(one), 8)
            .within(u128::from(lo), u128::from(hi));
        contains(k, u128::from(v)).then_some(k)
    }
    pub fn check(op: u8, a: [u8; 5], b: [u8; 5]) -> bool {
        let (Some(ka), Some(kb)) = (
            input(a[0], a[1], a[2], a[3], a[4]),
            input(b[0], b[1], b[2], b[3], b[4]),
        ) else {
            return true;
        };
        let (x, y) = (a[4], b[4]);
        let (k, v) = match op {
            0 => (combine("bvand", ka, kb), x & y),
            1 => (combine("bvor", ka, kb), x | y),
            2 => (combine("bvxor", ka, kb), x ^ y),
            3 => (combine("bvadd", ka, kb), x.wrapping_add(y)),
            4 => (combine("bvmul", ka, kb), x.wrapping_mul(y)),
            5 => (ka.not(), !x),
            6 => (add_carry(ka, kb.not(), false, true), x.wrapping_sub(y)),
            7 => (add_carry(ka.not(), Known::exact(0, 8), false, true), x.wrapping_neg()),
            8 => (shift("bvshl", ka, u128::from(y)), if y < 8 { x << y } else { 0 }),
            9 => (shift("bvlshr", ka, u128::from(y)), if y < 8 { x >> y } else { 0 }),
            10 => (shift("bvashr", ka, u128::from(y)), ((x as i8) >> y.min(7)) as u8),
            _ => return true,
        };
        contains(k, u128::from(v))
    }
}
"""


def transfer(directory: Path) -> str:
    (directory / "knownbits").mkdir(parents=True)
    shutil.copy(SRC / "knownbits" / "rewrite.rs", directory / "knownbits" / "rewrite.rs")
    (directory / "knownbits.rs").write_text((SRC / "knownbits.rs").read_text() + TRANSFER_CHECK)
    args = ", ".join(f"{n}: u8" for n in "op za oa la ha x zb ob lb hb y".split())
    return f"""//! Phage on Phage: known-bits transfer functions (src/knownbits.rs).
#![allow(dead_code)]
mod knownbits;
{shims()}
#[unsafe(no_mangle)]
pub fn phage_target({args}) -> bool {{
    knownbits::selfcheck::check(op, [za, oa, la, ha, x], [zb, ob, lb, hb, y])
}}
"""


def literal(_directory: Path) -> str:
    # Five characters from the parser's own alphabet: every 5-byte string
    # explodes into too many paths, these reach all of its branches.
    alphabet = "#xb(_ 1f"
    codes = ", ".join(str(ord(c)) for c in alphabet)
    picks = ", ".join(f"ALPHABET[usize::from(b{i} & 7)]" for i in range(5))
    args = ", ".join(f"b{i}: u8" for i in range(5))
    return f"""//! Phage on Phage: heap::literal never panics and reads `#x` digits
//! (5 characters from {alphabet!r}).
#![allow(dead_code)]
{shims()}
#[unsafe(no_mangle)]
pub fn phage_target({args}) -> bool {{
    const ALPHABET: [u8; 8] = [{codes}];
    let data = [{picks}];
    let Ok(text) = core::str::from_utf8(&data) else {{
        return true;
    }};
    let value = heap::literal(text);
    match text.strip_prefix("#x") {{
        Some(hex) if !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()) => {{
            value == Some(hex.bytes().fold(0u128, |v, b| {{
                v * 16 + u128::from((b as char).to_digit(16).unwrap_or(0))
            }}))
        }}
        _ => true,
    }}
}}
"""


def cache_full(directory: Path) -> str:
    """Run State::simplify twice through all production String-keyed maps."""
    (directory / "knownbits").mkdir(parents=True)
    shutil.copy(SRC / "knownbits.rs", directory / "knownbits.rs")
    shutil.copy(SRC / "knownbits" / "rewrite.rs", directory / "knownbits" / "rewrite.rs")
    return f"""//! Phage on Phage: repeated rewrites through State's real String caches.
#![allow(dead_code)]
mod knownbits;
{shims()}
#[unsafe(no_mangle)]
pub fn phage_target() -> bool {{
    let mut state = knownbits::State::default();
    state.declared("x", "(_ BitVec 8)");
    let term = "(bvand x (_ bv255 8))";
    let first = state.simplify(term, "(_ BitVec 8)");
    let second = state.simplify(term, "(_ BitVec 8)");
    first == "x" && second == "x"
}}
"""


def cache(directory: Path) -> str:
    """Exercise a production known-bits cache miss followed by a hit."""
    (directory / "knownbits").mkdir(parents=True)
    check = """
pub mod cache_selfcheck {
    use super::*;
    pub fn check(v: u8) -> bool {
        let mut state = State::default();
        state.widths.insert("x".to_owned(), 8);
        let mut s = Simplifier::new(&state.bodies, &state.widths,
            &mut state.known, &mut state.parsed);
        let node = atom("x");
        let mut contains = || {
            let Some(k) = s.known(&node) else { return false; };
            let result = u128::from(v);
            result & k.zero == 0 && result & k.one == k.one && k.lo <= result && result <= k.hi
        };
        contains() && contains()
    }
}
"""
    (directory / "knownbits.rs").write_text((SRC / "knownbits.rs").read_text() + check)
    shutil.copy(SRC / "knownbits" / "rewrite.rs", directory / "knownbits" / "rewrite.rs")
    return f"""//! Phage on Phage: a miss and hit in the real known-bits cache.
#![allow(dead_code)]
mod knownbits;
{shims()}
#[unsafe(no_mangle)]
pub fn phage_target(v: u8) -> bool {{
    knownbits::cache_selfcheck::check(v)
}}
"""


# name -> (generator, phage flags, mutants as (file, old, new, description))
HARNESSES = {
    "cache": (
        cache,
        ["--maxStates", "10000", "--queryTimeout", "3000"],
        [("knownbits.rs", "return *known;",
          "return known.map(|k| Known::exact(0, k.width));", "wrong value returned from cache")],
    ),
    "cache-full": (
        cache_full,
        ["--maxStates", "10000", "--queryTimeout", "3000"],
        [("knownbits/rewrite.rs", "if op == \"bvand\" { v == mask(w) } else { v == 0 }",
          "if op == \"bvand\" { v == 0 } else { v == 0 }", "wrong identity through cache")],
    ),
    "transfer": (
        transfer,
        [],
        [
            ("knownbits.rs", '"bvor" => Known::bits(a.zero & b.zero,', '"bvor" => Known::bits(a.zero | b.zero,', "bvor known zeros"),
            ("knownbits.rs", "Some(hi) => k.within(a.lo + b.lo, hi)", "Some(hi) => k.within(a.lo + b.lo + 1, hi)", "bvadd lower bound"),
            ("knownbits.rs", "(x.one >> c) | if x.one & sign != 0 { high } else { 0 },", "(x.one >> c) | if x.zero & sign != 0 { high } else { 0 },", "bvashr sign fill"),
            ("knownbits.rs", ".wrapping_add(u128::from(carry_one))", ".wrapping_add(1)", "add_carry carry-in"),
            ("knownbits.rs", "mask((a.trailing_zeros() + b.trailing_zeros()).min(w))", "mask((a.trailing_zeros() + b.trailing_zeros() + 1).min(w))", "bvmul trailing zeros"),
            ("knownbits.rs", ".within(m - self.hi, m - self.lo)", ".within(m - self.hi + 1, m - self.lo)", "not lower bound"),
        ],
    ),
    "literal": (
        literal,
        ["--maxStates", "300000"],
        [("check.rs", "u128::from_str_radix(hex, 16)", "u128::from_str_radix(hex, 10)", "hex digits read as decimal")],
    ),
}


def verdict(phage: str, directory: Path, flags: list[str], timeout: int) -> tuple[str, float]:
    started = time.monotonic()
    run = subprocess.Popen(
        [phage, "check", "check.rs", *flags],
        cwd=directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    try:
        output, _ = run.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(run.pid, signal.SIGKILL)
        output, _ = run.communicate()
        (directory / "phage.log").write_text(output + "\nrunner timeout; inconclusive\n")
        return "unknown-timeout", time.monotonic() - started
    lines = output.splitlines()
    (directory / "phage.log").write_text(output)
    status = next((l.split(":")[0] for l in lines if l.startswith(("proved", "counterexample", "unknown"))), "error")
    return status, time.monotonic() - started


def main() -> int:
    parser = argparse.ArgumentParser(description="Phage on Phage: verify parts of Phage with Phage.")
    parser.add_argument("--phage", default=str(REPO / "target" / "release" / "phage"))
    parser.add_argument("--only", nargs="*", choices=sorted(HARNESSES))
    parser.add_argument("--timeout", type=int, default=3600, help="seconds per attempt")
    parser.add_argument(
        "--work",
        type=Path,
        default=WORK,
        help="parent directory for retained per-run work",
    )
    options = parser.parse_args()
    failures = 0
    work = options.work.resolve() / str(time.time_ns())
    records = []
    for name in options.only or ["transfer", "cache", "literal"]:
        generate, flags, mutants = HARNESSES[name]
        base = work / name
        original = base / "original"
        original.mkdir(parents=True)
        (original / "check.rs").write_text(generate(original))
        cases = [("original", original, "proved")]
        for index, (file, old, new, description) in enumerate(mutants):
            directory = base / f"mutant{index}"
            shutil.copytree(original, directory)
            target = directory / file
            text = target.read_text()
            if text.count(old) != 1:
                print(f"{name}: stale mutant '{description}' (text not found once in {file})")
                failures += 1
                continue
            target.write_text(text.replace(old, new))
            cases.append((description, directory, "counterexample"))
        for description, directory, expected in cases:
            status, seconds = verdict(options.phage, directory, flags, options.timeout)
            mark = "ok" if status == expected else "FAIL"
            failures += status != expected
            records.append({"harness": name, "case": description, "status": status,
                            "expected": expected, "wallSeconds": seconds,
                            "directory": str(directory)})
            (work / "RESULT.json").write_text(json.dumps(records, indent=2) + "\n")
            print(f"{name}: {description}: {status} (expected {expected}) {seconds:.1f}s {mark}", flush=True)
    print(f"selfcheck: {'all passed' if failures == 0 else f'{failures} failed'}; work in {work}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
