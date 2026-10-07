use super::*;

/// Every `Known` of `width` bits (each bit zero, one or unknown), plus a
/// narrowed-bounds variant of each.
fn all_known(width: u32) -> Vec<Known> {
    let mut out = Vec::new();
    for code in 0..3u32.pow(width) {
        let (mut zero, mut one, mut c) = (0u128, 0u128, code);
        for bit in 0..width {
            match c % 3 {
                0 => zero |= 1 << bit,
                1 => one |= 1 << bit,
                _ => {}
            }
            c /= 3;
        }
        let k = Known::bits(zero, one, width);
        out.push(k);
        let narrowed = k.within(
            k.lo + u128::from(code % 3),
            k.hi.saturating_sub(u128::from(code % 2)),
        );
        if narrowed != k && !members(narrowed).is_empty() {
            out.push(narrowed);
        }
    }
    out
}

fn members(k: Known) -> Vec<u128> {
    (0..=mask(k.width))
        .filter(|v| v & k.zero == 0 && v & k.one == k.one && (k.lo..=k.hi).contains(v))
        .collect()
}

fn concrete(op: &str, a: u128, b: u128, width: u32) -> u128 {
    let m = mask(width);
    let r = match op {
        "bvadd" => a.wrapping_add(b),
        "bvsub" => a.wrapping_sub(b),
        "bvmul" => a.wrapping_mul(b),
        "bvand" => a & b,
        "bvor" => a | b,
        "bvxor" => a ^ b,
        "bvshl" => {
            if b >= u128::from(width) {
                0
            } else {
                a << b
            }
        }
        "bvlshr" => {
            if b >= u128::from(width) {
                0
            } else {
                a >> b
            }
        }
        "bvurem" => {
            if b == 0 {
                a
            } else {
                a % b
            }
        }
        _ => unreachable!(),
    };
    r & m
}

/// Each transfer function, exhaustively at 4 bits: every value consistent
/// with the inputs' known bits yields a result consistent with the output.
#[test]
fn transfer_functions_are_sound() {
    // Every pair at 3 bits; a sample of pairs at 4 bits.
    for (width, stride) in [(3, 1), (4, 7)] {
        transfer_functions_at(width, stride);
    }
}

fn transfer_functions_at(width: u32, stride: usize) {
    let knowns = all_known(width);
    let bodies = BTreeMap::new();
    let mut widths = BTreeMap::new();
    widths.insert("a".to_string(), width);
    widths.insert("b".to_string(), width);
    for op in [
        "bvadd", "bvsub", "bvmul", "bvand", "bvor", "bvxor", "bvshl", "bvlshr", "bvurem",
    ] {
        for (i, ka) in knowns.iter().enumerate() {
            // A sample of right operands keeps the test fast but varied.
            for kb in knowns.iter().skip(i % stride).step_by(stride) {
                // Through the simplifier, as the engine computes it; shift
                // amounts and divisors are literals there.
                let right = match op {
                    "bvshl" | "bvlshr" | "bvurem" => match kb.value() {
                        Some(c) => crate::value::bv(c, width),
                        None => continue,
                    },
                    _ => "b".to_string(),
                };
                let mut cache = BTreeMap::new();
                cache.insert("a".to_string(), Some(*ka));
                cache.insert("b".to_string(), Some(*kb));
                let mut parsed = BTreeMap::new();
                let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
                let term = parse(&format!("({op} a {right})")).unwrap();
                let result = s.known(&term).unwrap();
                for x in members(*ka) {
                    for y in members(*kb) {
                        let r = concrete(op, x, y, width);
                        assert!(
                            r & result.zero == 0
                                && r & result.one == result.one
                                && (result.lo..=result.hi).contains(&r),
                            "{op} {ka:?} {kb:?}: {x} {y} -> {r} not in {result:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn flag_conditions_of_a_byte_pair_discharge() {
    let mut bodies = BTreeMap::new();
    bodies.insert("d16".to_string(), "((_ zero_extend 56) b)".to_string());
    bodies.insert("d17".to_string(), "(bvshl d16 (_ bv8 64))".to_string());
    bodies.insert("d19".to_string(), "((_ zero_extend 56) a)".to_string());
    let mut widths = BTreeMap::new();
    widths.insert("a".to_string(), 8);
    widths.insert("b".to_string(), 8);
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    let condition = "(and (bvult (_ bv8 64) (_ bv64 64)) (= (bvlshr (bvshl d16 (_ bv8 64)) (_ bv8 64)) d16) (= (bvashr (bvshl d16 (_ bv8 64)) (_ bv8 64)) d16) (= (bvand d17 d19) (_ bv0 64)))";
    assert_eq!(render(&s.boolean(&parse(condition).unwrap())), "true");
    // Negative controls: a 64-bit `b` can overflow and overlap.
    widths.insert("w".to_string(), 64);
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    for open in [
        "(= (bvlshr (bvshl w (_ bv8 64)) (_ bv8 64)) w)",
        "(= (bvand w d19) (_ bv0 64))",
        "(bvuge (bvadd w d19) w)",
    ] {
        assert_eq!(render(&s.boolean(&parse(open).unwrap())), open);
    }
    // Shift amounts at the top of u32 must not wrap into a "true".
    for huge in [
        "(= (bvashr (bvshl w (_ bv4294967295 64)) (_ bv4294967295 64)) w)",
        "(= (bvlshr (bvshl w (_ bv4294967295 64)) (_ bv4294967295 64)) w)",
    ] {
        // Both shifts give 0, so the identity holds only for w = 0.
        assert_eq!(
            render(&s.boolean(&parse(huge).unwrap())),
            "(= (_ bv0 64) w)"
        );
    }
    // In bounds: 16 + (h & 15) <= 48 for any h.
    let bound = "(and (bvule (bvadd (_ bv16 64) (bvand w (_ bv15 64))) (_ bv48 64)) (bvuge (bvadd (_ bv16 64) (bvand w (_ bv15 64))) (_ bv16 64)))";
    assert_eq!(render(&s.boolean(&parse(bound).unwrap())), "true");
    assert_eq!(
        render(&s.boolean(
            &parse("(bvule (bvadd (_ bv40 64) (bvand w (_ bv15 64))) (_ bv48 64))").unwrap()
        )),
        "(bvule (bvadd (_ bv40 64) (bvand w (_ bv15 64))) (_ bv48 64))"
    );
}

#[test]
fn reads_resolve_over_literal_and_uniform_stores() {
    let mut bodies = BTreeMap::new();
    let base = "((as const (Array (_ BitVec 64) Bool)) false)";
    bodies.insert(
        "m1".to_string(),
        format!("(store (store {base} (_ bv16 64) true) (bvadd (_ bv16 64) (_ bv1 64)) x)"),
    );
    bodies.insert(
        "m2".to_string(),
        format!("(store (store {base} i false) j false)"),
    );
    let widths = BTreeMap::new();
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    let read = |s: &mut Simplifier, t: &str| render(&s.boolean(&parse(t).unwrap()));
    assert_eq!(read(&mut s, "(select m1 (_ bv16 64))"), "true");
    assert_eq!(read(&mut s, "(select m1 (_ bv17 64))"), "x");
    assert_eq!(read(&mut s, "(select m1 (_ bv3 64))"), "false");
    // Symbolic stores of the base value everywhere: any read is false.
    assert_eq!(read(&mut s, "(select m2 k)"), "false");
    // A symbolic index whose range literal stores cover: memset flags.
    let mut covered = base.to_string();
    for i in 16..48 {
        covered = format!("(store {covered} (_ bv{i} 64) true)");
    }
    bodies.insert(
        "m4".to_string(),
        format!("(store {covered} (bvadd (_ bv8 64) h) true)"),
    );
    let mut widths = BTreeMap::new();
    widths.insert("h".to_string(), 64);
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    let probe = "(select m4 (bvadd (_ bv16 64) (bvand h (_ bv15 64))))";
    assert_eq!(read(&mut s, probe), "true");
    // Negative controls: the range leaves the covered bytes; an aliasing
    // store of another value.
    let wide = "(select m4 (bvadd (_ bv40 64) (bvand h (_ bv15 64))))";
    assert_eq!(read(&mut s, wide), wide);
    bodies.insert("m5".to_string(), format!("(store {covered} h false)"));
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    let aliased = "(select m5 (bvadd (_ bv16 64) (bvand h (_ bv15 64))))";
    assert_eq!(read(&mut s, aliased), aliased);
    // Negative control: a symbolic store of another value may alias.
    bodies.insert("m3".to_string(), format!("(store {base} i true)"));
    let mut cache = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
    assert_eq!(
        read(&mut s, "(select m3 (_ bv3 64))"),
        "(select m3 (_ bv3 64))"
    );
}

/// Random terms over symbols: each rewrite is equivalent according to Z3.
#[test]
fn rewrites_agree_with_z3() {
    let mut solver = crate::solver::Solver::new(5000).expect("Z3 required");
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut widths = BTreeMap::new();
    for name in ["p", "q"] {
        solver.declare(name, "(_ BitVec 16)").unwrap();
        widths.insert(name.to_string(), 16);
    }
    let bodies = BTreeMap::new();
    let leaf = |r: u64| -> String {
        match r % 6 {
            0 => "p".into(),
            1 => "q".into(),
            2 => format!(
                "((_ zero_extend 8) ((_ extract 7 0) {}))",
                if r & 8 == 0 { "p" } else { "q" }
            ),
            3 => format!("(bvand p (_ bv{} 16))", (r >> 8) & 0xffff),
            4 => format!("(bvshl q (_ bv{} 16))", (r >> 8) % 18),
            _ => format!("(_ bv{} 16)", (r >> 8) & 0xffff),
        }
    };
    let ops = [
        "bvadd", "bvsub", "bvand", "bvor", "bvxor", "bvmul", "bvlshr", "bvurem",
    ];
    let compares = [
        "=", "bvult", "bvule", "bvugt", "bvuge", "bvslt", "bvsle", "bvsgt", "bvsge",
    ];
    let mut decided = 0;
    for round in 0..300 {
        let a = format!(
            "({} {} {})",
            ops[next() as usize % ops.len()],
            leaf(next()),
            leaf(next())
        );
        let b = if round % 2 == 0 {
            leaf(next())
        } else {
            format!("(_ bv{} 16)", next() & 0xffff)
        };
        let mut term = format!("({} {a} {b})", compares[next() as usize % compares.len()]);
        if round % 3 == 0 {
            // The `shl` flag identities, on operands with and without
            // known high bits.
            let x = leaf(next());
            let shr = if round % 2 == 0 { "bvlshr" } else { "bvashr" };
            let c = format!("(_ bv{} 16)", next() % 12);
            term = format!("(= ({shr} (bvshl {x} {c}) {c}) {x})");
        }
        let mut cache = BTreeMap::new();
        let mut parsed = BTreeMap::new();
        let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
        let simplified = render(&s.boolean(&parse(&term).unwrap()));
        if simplified == "true" || simplified == "false" {
            decided += 1;
        }
        // The bit-vector rewrite of an `ite`/identity-wrapped operand.
        let wrapped = match round % 4 {
            0 => format!("(ite (bvult (_ bv3 16) (_ bv9 16)) {a} q)"),
            1 => format!("(bvadd {a} (bvand p (_ bv0 16)))"),
            2 => format!("(bvand (_ bv65535 16) {a})"),
            _ => format!("(bvurem {a} (_ bv8 16))"),
        };
        let bits = render(&s.bitvector(&parse(&wrapped).unwrap()));
        assert_eq!(
            solver
                .check(&[], &format!("(not (= {wrapped} {bits}))"))
                .unwrap(),
            crate::solver::Sat::No,
            "{wrapped} simplified to {bits}"
        );
        let name = format!("t{round}");
        solver.declare(&name, "Bool").unwrap();
        assert_eq!(
            solver
                .check(
                    &[format!("(= {name} {term})")],
                    &format!("(not (= {name} {simplified}))")
                )
                .unwrap(),
            crate::solver::Sat::No,
            "{term} simplified to {simplified}"
        );
    }
    // The rewrites must actually fire, or this test proves nothing.
    assert!(decided > 20, "only {decided} terms decided");
}

/// Random store chains and reads: each resolved read is equivalent to the
/// original `select` according to Z3.
#[test]
fn reads_agree_with_z3() {
    let mut solver = crate::solver::Solver::new(5000).expect("Z3 required");
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut widths = BTreeMap::new();
    for name in ["p", "q"] {
        solver.declare(name, "(_ BitVec 8)").unwrap();
        widths.insert(name.to_string(), 8);
    }
    solver.declare("b", "Bool").unwrap();
    let index = |r: u64| -> String {
        match r % 5 {
            0 => format!("(_ bv{} 8)", (r >> 4) % 12),
            1 => "(bvand p (_ bv3 8))".into(),
            2 => "(bvadd (_ bv4 8) (bvand q (_ bv3 8)))".into(),
            3 => "p".into(),
            _ => format!("(_ bv{} 8)", 4 + (r >> 4) % 4),
        }
    };
    let value = |r: u64| ["true", "false", "b"][(r % 3) as usize];
    let mut resolved = 0;
    for round in 0..300 {
        let base = format!("((as const (Array (_ BitVec 8) Bool)) {})", value(next()));
        let mut chain = base;
        let mut bodies = BTreeMap::new();
        for step in 0..1 + next() % 6 {
            chain = format!("(store {chain} {} {})", index(next()), value(next()));
            if step == 1 {
                bodies.insert(format!("m{round}"), chain.clone());
                solver
                    .define(&format!("m{round}"), "(Array (_ BitVec 8) Bool)", &chain)
                    .unwrap();
                chain = format!("m{round}");
            }
        }
        let term = format!("(select {chain} {})", index(next()));
        let mut cache = BTreeMap::new();
        let mut parsed = BTreeMap::new();
        let mut s = Simplifier::new(&bodies, &widths, &mut cache, &mut parsed);
        let simplified = render(&s.boolean(&parse(&term).unwrap()));
        if simplified != term {
            resolved += 1;
        }
        assert_eq!(
            solver
                .check(&[], &format!("(not (= {term} {simplified}))"))
                .unwrap(),
            crate::solver::Sat::No,
            "{term} read as {simplified}"
        );
    }
    assert!(resolved > 60, "only {resolved} reads resolved");
}
