//! Positive, negative and unknown controls for the interpreter, independent
//! of rustc optimizations. These tests require the real Z3 solver.

use super::Engine;
use crate::ir;
use std::rc::Rc;

/// An engine with `text` loaded as the checked module, and its entry.
fn load(text: &str, timeout: u64, unwind: usize, states: usize) -> (Engine, ir::Function) {
    let module = ir::Module::new("user", text.to_owned(), true).expect("fixture must parse");
    let function = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(timeout, unwind, states).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    (engine, function)
}
fn verify(parameters: &str, blocks: &str, unwind: usize, states: usize) -> String {
    let source = format!("define i1 @phage_target({parameters}) {{\n{blocks}\n}}\n");
    let (mut engine, function) = load(&source, 5000, unwind, states);
    engine
        .verify(&function)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
fn expect(parameters: &str, blocks: &str, status: &str) {
    assert_eq!(verify(parameters, blocks, 16, 1000), status);
}

#[test]
fn tautology_and_false_control() {
    expect(
        "i8 %x",
        "start:\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "proved",
    );
    expect(
        "i8 %x",
        "start:\n %v = icmp eq i8 %x, 0\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn unsigned_and_signed_add_overflow_are_poison() {
    for (flag, bound) in [("nuw", 255), ("nsw", 127)] {
        expect(
            "i8 %x",
            &format!(
                "start:\n %y = add {flag} i8 %x, 1\n %c = icmp eq i8 %x, {bound}\n br i1 %c, label %bad, label %ok\nbad:\n %v = icmp eq i8 %y, %y\n ret i1 %v\nok:\n ret i1 true"
            ),
            "counterexample",
        );
    }
}
#[test]
fn wrapping_add_is_defined() {
    expect(
        "i8 %x",
        "start:\n %y = add i8 %x, 1\n %z = sub i8 %y, 1\n %c = icmp eq i8 %z, %x\n ret i1 %c",
        "proved",
    );
}
#[test]
fn checked_overflow_intrinsic_preserves_both_results() {
    expect(
        "i8 %x",
        "start:\n %a = call { i8, i1 } @llvm.uadd.with.overflow.i8(i8 %x, i8 1)\n %o = extractvalue { i8, i1 } %a, 1\n %c = icmp eq i8 %x, 255\n %v = icmp eq i1 %o, %c\n ret i1 %v",
        "proved",
    );
}
#[test]
fn signed_multiply_overflow_is_poison() {
    expect(
        "",
        "start:\n %x = mul nsw i8 64, 2\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn unsigned_subtract_underflow_is_poison() {
    expect(
        "",
        "start:\n %x = sub nuw i8 0, 1\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn shift_width_and_exact_shift_are_checked() {
    for instruction in [
        "shl i8 1, 8",
        "lshr exact i8 3, 1",
        "shl nsw i8 64, 1",
        "shl nuw i8 128, 1",
    ] {
        expect(
            "",
            &format!("start:\n %x = {instruction}\n %v = icmp eq i8 %x, %x\n ret i1 %v"),
            "counterexample",
        );
    }
}
#[test]
fn divide_by_zero_is_poison() {
    expect(
        "",
        "start:\n %x = udiv i8 4, 0\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn disjoint_and_same_sign_flags_are_checked() {
    expect(
        "",
        "start:\n %x = or disjoint i8 3, 2\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
    expect(
        "",
        "start:\n %v = icmp samesign ult i8 128, 1\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn nonnegative_extension_is_checked() {
    expect(
        "",
        "start:\n %x = zext nneg i8 128 to i16\n %v = icmp eq i16 %x, %x\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn unselected_poison_is_not_observable() {
    expect(
        "",
        "start:\n %v = select i1 true, i1 true, i1 poison\n ret i1 %v",
        "proved",
    );
    expect(
        "",
        "start:\n %v = select i1 poison, i1 true, i1 true\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn observed_undef_cannot_be_proved_as_stable() {
    // Never proved: an undef result may be false (undefined bytes).
    expect("", "start:\n ret i1 undef", "counterexample");
}
#[test]
fn simultaneous_loop_phi_assignment() {
    expect(
        "",
        "start:\n br label %loop\nloop:\n %a = phi i8 [ 1, %start ], [ %b, %loop ]\n %b = phi i8 [ 2, %start ], [ %a, %loop ]\n %i = phi i8 [ 0, %start ], [ %next, %loop ]\n %next = add i8 %i, 1\n %c = icmp ult i8 %i, 1\n br i1 %c, label %loop, label %end\nend:\n %ok = icmp eq i8 %b, 1\n ret i1 %ok",
        "proved",
    );
}
#[test]
fn feasible_exploration_limits_are_unknown() {
    assert_eq!(
        verify(
            "",
            "start:\n br label %loop\nloop:\n br label %loop",
            2,
            100
        ),
        "unknown"
    );
    assert_eq!(
        verify("", "start:\n br label %end\nend:\n ret i1 true", 10, 1),
        "unknown"
    );
}
#[test]
fn unsupported_reachable_code_blocks_proof() {
    expect(
        "i1 %c",
        "start:\n br i1 %c, label %a, label %b\na:\n %x = call i1 @external()\n ret i1 %x\nb:\n ret i1 true",
        "unknown",
    );
    expect(
        "",
        "start:\n %x = freeze i1 poison\n ret i1 true",
        "unknown",
    );
}
#[test]
fn unreachable_unsupported_code_does_not_block_proof() {
    expect(
        "",
        "start:\n br i1 true, label %ok, label %bad\nbad:\n %x = call i1 @external()\n ret i1 %x\nok:\n ret i1 true",
        "proved",
    );
}
#[test]
fn initialized_little_endian_memory_roundtrip() {
    expect(
        "i16 %x",
        "start:\n %p = alloca i16, align 2\n store i16 %x, ptr %p, align 2\n %lo = load i8, ptr %p, align 1\n %q = getelementptr inbounds i8, ptr %p, i64 1\n %hi = load i8, ptr %q, align 1\n %l = zext i8 %lo to i16\n %h = zext i8 %hi to i16\n %hs = shl i16 %h, 8\n %y = or i16 %l, %hs\n %v = icmp eq i16 %y, %x\n ret i1 %v",
        "proved",
    );
}
#[test]
fn uninitialized_load_is_observable_poison() {
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %x = load i8, ptr %p, align 1\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
}
#[test]
fn one_past_read_and_write_are_invalid() {
    for operation in [
        "%x = load i8, ptr %q, align 1",
        "store i8 1, ptr %q, align 1",
    ] {
        expect(
            "",
            &format!(
                "start:\n %p = alloca i8, align 1\n %q = getelementptr inbounds i8, ptr %p, i64 1\n {operation}\n ret i1 true"
            ),
            "counterexample",
        );
    }
}
#[test]
fn misaligned_read_is_invalid() {
    expect(
        "",
        "start:\n %p = alloca [4 x i8], align 2\n %q = getelementptr inbounds i8, ptr %p, i64 1\n %x = load i16, ptr %q, align 2\n ret i1 true",
        "counterexample",
    );
}
#[test]
fn ended_lifetime_load_is_poison_until_observed() {
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n store i8 7, ptr %p, align 1\n call void @llvm.lifetime.end.p0(i64 1, ptr %p)\n %x = load i8, ptr %p, align 1\n ret i1 true",
        "proved",
    );
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n store i8 7, ptr %p, align 1\n call void @llvm.lifetime.end.p0(i64 1, ptr %p)\n %x = load i8, ptr %p, align 1\n %ok = icmp eq i8 %x, 7\n ret i1 %ok",
        "counterexample",
    );
}
#[test]
fn stack_address_is_nonnull_and_choices_resolve_by_path() {
    expect(
        "i1 %c",
        "start:\n %p = alloca i8, align 1\n store i8 7, ptr %p, align 1\n %q = select i1 %c, ptr %p, ptr null\n %isnull = icmp eq ptr %q, null\n br i1 %isnull, label %no, label %yes\nyes:\n %x = load i8, ptr %q, align 1\n %v = icmp eq i8 %x, 7\n ret i1 %v\nno:\n ret i1 true",
        "proved",
    );
}
#[test]
fn pointer_choices_split_the_path() {
    // Each branch of the split reads uninitialized memory (UB with noundef).
    expect(
        "i1 %c",
        "start:\n %p = alloca i8, align 1\n %q = alloca i8, align 1\n %r = select i1 %c, ptr %p, ptr %q\n %x = load i8, ptr %r, align 1, !noundef !0\n ret i1 true",
        "counterexample",
    );
    // With both objects initialized, each branch reads its own object.
    expect(
        "i1 %c",
        "start:\n %p = alloca i8, align 1\n %q = alloca i8, align 1\n store i8 1, ptr %p, align 1\n store i8 2, ptr %q, align 1\n %r = select i1 %c, ptr %p, ptr %q\n %x = load i8, ptr %r, align 1\n %w = select i1 %c, i8 1, i8 2\n %v = icmp eq i8 %x, %w\n ret i1 %v",
        "proved",
    );
}
#[test]
fn constants_and_pointer_integer_casts() {
    let text = "@s = private unnamed_addr constant [1 x i8] c\"A\", align 1\ndefine i1 @phage_target() {\nstart:\n %x = load i8, ptr @s, align 1\n %a = ptrtoint ptr @s to i64\n %nz = icmp ne i64 %a, 0\n %ok = icmp eq i8 %x, 65\n %both = and i1 %nz, %ok\n ret i1 %both\n}\n";
    let (mut engine, function) = load(text, 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "proved");
}
#[test]
fn solver_resource_exhaustion_never_becomes_proof() {
    let function = ir::function("define i1 @phage_target(i8 %x) {\nstart:\n %a = mul i8 %x, %x\n %v = icmp eq i8 %a, 17\n ret i1 %v\n}\n", "phage_target").unwrap();
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    // Every solver is exhausted: the incremental one by rlimit, racers off.
    engine.solver.portfolio = false;
    engine.solver.send("(set-option :rlimit 1)").unwrap();
    match engine.verify(&function) {
        Ok(result) => assert_eq!(result.status, "unknown"),
        Err(error) => assert!(
            error.contains("canceled") || error.contains("unknown"),
            "unexpected solver failure: {error}"
        ),
    }
}

#[test]
fn merged_global_addresses_are_not_assumed_distinct() {
    let text = "@a = private unnamed_addr constant [1 x i8] c\"A\", align 1\n@b = private unnamed_addr constant [1 x i8] c\"A\", align 1\ndefine i1 @phage_target() {\nstart:\n %v = icmp ne ptr @a, @b\n ret i1 %v\n}\n";
    let (mut engine, function) = load(text, 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "counterexample");
}
#[test]
fn live_stack_regions_are_disjoint() {
    expect(
        "",
        "start:\n %a = alloca i8, align 1\n %b = alloca i8, align 1\n %v = icmp ne ptr %a, %b\n ret i1 %v",
        "proved",
    );
}

#[test]
fn undef_phi_is_not_one_stable_symbol() {
    expect(
        "",
        "start:\n br label %end\nend:\n %x = phi i8 [ undef, %start ]\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        // Two uses of undef may differ: each observation is a fresh value.
        "counterexample",
    );
}

#[test]
fn unused_zero_division_is_still_invalid() {
    for op in ["udiv", "urem"] {
        expect(
            "",
            &format!("start:\n %x = {op} i8 4, 0\n ret i1 true"),
            "counterexample",
        );
    }
}
#[test]
fn uninitialized_loads_follow_noundef_metadata() {
    // With `!noundef`, reading undefined bytes is undefined behavior even
    // when the value is unused.
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %x = load i8, ptr %p, align 1, !noundef !0\n ret i1 true",
        "counterexample",
    );
    // Without it the value is poison: unused, nothing is wrong (a copy of
    // MaybeUninit bytes); observed, it fails where it is used.
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %x = load i8, ptr %p, align 1\n ret i1 true",
        "proved",
    );
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %x = load i8, ptr %p, align 1\n %v = icmp eq i8 %x, %x\n ret i1 %v",
        "counterexample",
    );
    // Copying undefined bytes keeps them undefined.
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %q = alloca i8, align 1\n %x = load i8, ptr %p, align 1\n store i8 %x, ptr %q, align 1\n %y = load i8, ptr %q, align 1\n %v = icmp eq i8 %y, 0\n br i1 %v, label %a, label %b\na:\n ret i1 true\nb:\n ret i1 true",
        "counterexample",
    );
    // Storing undef leaves the bytes undefined rather than failing.
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n store i8 undef, ptr %p, align 1\n %x = load i8, ptr %p, align 1\n ret i1 true",
        "proved",
    );
    // Dereferencing null is undefined behavior; a zero-length copy is not.
    expect(
        "",
        "start:\n store i8 1, ptr null, align 1\n ret i1 true",
        "counterexample",
    );
    expect(
        "",
        "start:\n call void @llvm.memcpy.p0.p0.i64(ptr null, ptr null, i64 0, i1 false)\n ret i1 true",
        "proved",
    );
    // Certainly undefined pointer bytes read as a poison pointer.
    expect(
        "",
        "start:\n %p = alloca ptr, align 8\n %q = load ptr, ptr %p, align 8\n store i8 1, ptr %q, align 1\n ret i1 true",
        "counterexample",
    );
}

#[test]
fn unbounded_wrapping_pointer_can_be_null() {
    // GEP without inbounds can wrap an address. It must not receive the
    // nonnull shortcut that is valid only for a defined bounded pointer.
    expect(
        "i64 %index",
        "start:\n %p = alloca i8, align 1\n %q = getelementptr i8, ptr %p, i64 %index\n %v = icmp ne ptr %q, null\n ret i1 %v",
        "counterexample",
    );
}

#[test]
fn poison_pointer_comparison_does_not_hide_invalid_gep() {
    expect(
        "",
        "start:\n %p = alloca i8, align 1\n %q = getelementptr inbounds nuw i8, ptr %p, i64 2\n %v = icmp ne ptr %q, null\n ret i1 %v",
        "counterexample",
    );
}

#[test]
fn constant_gep_operands_keep_bounds_and_flags() {
    let global = "@s = private constant [2 x i8] c\"AB\", align 1\n";
    let select = "define i1 @phage_target(i1 %pick) {\nstart:\n %p = select i1 %pick, ptr getelementptr inbounds nuw (i8, ptr @s, i64 1), ptr @s\n %x = load i8, ptr %p, align 1\n %e = select i1 %pick, i8 66, i8 65\n %ok = icmp eq i8 %x, %e\n ret i1 %ok\n}\n";
    let (mut engine, function) = load(&format!("{global}{select}"), 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "proved");

    // One-past is a valid inbounds pointer but cannot be dereferenced.
    let one_past = select.replace("i64 1", "i64 2");
    let (mut engine, function) = load(&format!("{global}{one_past}"), 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "counterexample");
    // Past one-past makes the constant expression itself poison.
    let past = select.replace("i64 1", "i64 3");
    let (mut engine, function) = load(&format!("{global}{past}"), 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "counterexample");
    // Other constant GEP forms stay unknown rather than gaining semantics.
    let unsupported = select.replace("inbounds nuw (i8", "inbounds nuw (i16");
    let (mut engine, function) = load(&format!("{global}{unsupported}"), 5000, 16, 1000);
    assert_eq!(engine.verify(&function).unwrap().status, "unknown");
}

#[test]
fn absorbing_boolean_operations_mask_pointer_undef_but_not_poison() {
    let property = |pointer: &str, operation: &str| {
        format!(
            "start:\n br i1 %guard, label %go, label %done\ngo:\n %u = icmp eq ptr {pointer}, null\n %ok = {operation} i1 %guard, %u\n ret i1 %ok\ndone:\n ret i1 true"
        )
    };
    assert_eq!(
        verify("i1 %guard", &property("undef", "or"), 16, 1000),
        "proved"
    );
    assert_eq!(
        verify(
            "",
            "start:\n %u = icmp eq ptr undef, null\n %r = and i1 false, %u\n %ok = icmp eq i1 %r, false\n ret i1 %ok",
            16,
            1000
        ),
        "proved"
    );
    // The comparison can be false when its undef result is observable.
    assert_eq!(
        verify(
            "",
            "start:\n %u = icmp eq ptr undef, null\n ret i1 %u",
            16,
            1000
        ),
        "counterexample"
    );
    // Poison is never masked by ordinary bitwise instructions.
    assert_eq!(
        verify("i1 %guard", &property("poison", "or"), 16, 1000),
        "counterexample"
    );
    // Storing an undef pointer writes undefined bytes without immediate UB.
    assert_eq!(
        verify(
            "",
            "start:\n %slot = alloca ptr, align 8\n store ptr undef, ptr %slot, align 8\n ret i1 true",
            16,
            1000
        ),
        "proved"
    );
}

#[test]
fn absorbing_boolean_operations_mask_integer_undef_but_not_poison() {
    // rustc's `Option<&u16>::copied().filter(|v| *v != 0).unwrap_or(d)`: the
    // None path compares an undef payload, masks it with `and false`, and the
    // select then returns the default. Defined on every path.
    let filter = |payload: &str| {
        format!(
            "start:\n br i1 %some, label %load, label %join\nload:\n br label %join\njoin:\n %v = phi i16 [ %x, %load ], [ {payload}, %start ]\n %nz = icmp ne i16 %v, 0\n %take = and i1 %some, %nz\n %r = select i1 %take, i16 %v, i16 7\n %ok = icmp ne i16 %r, 0\n ret i1 %ok"
        )
    };
    assert_eq!(
        verify("i1 %some, i16 %x", &filter("undef"), 16, 1000),
        "proved"
    );
    // The same shape with poison is still poison on the None path.
    assert_eq!(
        verify("i1 %some, i16 %x", &filter("poison"), 16, 1000),
        "counterexample"
    );
    // An undef integer comparison that reaches the result stays observable.
    assert_eq!(
        verify(
            "",
            "start:\n %u = icmp eq i16 undef, 0\n ret i1 %u",
            16,
            1000
        ),
        "counterexample"
    );
}

/// LLVM 21 indexes a field of an array element with one multi-index GEP
/// (`%G, ptr %a, i64 %i, i32 1`); it must equal the byte offset i*24+16, and
/// an index past the array must make the inbounds result poison.
#[test]
fn multi_index_struct_gep_matches_the_byte_layout() {
    let module = |limit: u32| {
        format!(
            "%G = type {{ [8 x i16], i64 }}\ndefine i1 @phage_target(i64 %i) {{\nstart:\n %c = icmp ult i64 %i, {limit}\n br i1 %c, label %go, label %done\ngo:\n %a = alloca [768 x i8], align 8\n %p = getelementptr inbounds %G, ptr %a, i64 %i, i32 1\n store i64 5, ptr %p, align 8\n %m = mul i64 %i, 24\n %o = add i64 %m, 16\n %q = getelementptr inbounds i8, ptr %a, i64 %o\n %v = load i64, ptr %q, align 8\n %ok = icmp eq i64 %v, 5\n ret i1 %ok\ndone:\n ret i1 true\n}}\n"
        )
    };
    let run = |text: String| {
        let (mut engine, function) = load(&text, 5000, 16, 1000);
        engine
            .verify(&function)
            .map(|v| v.status)
            .unwrap_or_else(|e| format!("unknown: {e}"))
    };
    assert_eq!(run(module(32)), "proved");
    // i = 32 lands past the 768-byte allocation: poison pointer, store is UB.
    assert_eq!(run(module(33)), "counterexample");
    // A field index that does not exist stays unknown, never proved.
    let missing = module(32).replace("i64 %i, i32 1", "i64 %i, i32 2");
    assert!(run(missing).starts_with("unknown"));
}

/// rustc's SROA rebuilds an enum from its tag byte and a partly
/// uninitialized payload (`zext`, `shl nuw 8`, `or disjoint`, `trunc`); the
/// tag stays defined. Shifting an undefined byte out under `nuw`, or reading a
/// payload byte, is still undefined.
#[test]
fn enum_tag_survives_bytewise_rebuild_of_an_uninitialized_payload() {
    let program = |shift: u32, read: u32| {
        format!(
            "start:\n %a = alloca [8 x i8], align 8\n store i8 2, ptr %a, align 8\n %t = load i8, ptr %a, align 8\n %p = getelementptr inbounds nuw i8, ptr %a, i64 1\n %r = load i56, ptr %p, align 1\n %z = zext i56 %r to i64\n %s = shl nuw i64 %z, {shift}\n %tz = zext i8 %t to i64\n %o = or disjoint i64 %s, %tz\n %tr = trunc i64 %o to i{read}\n %c = icmp eq i{read} %tr, 2\n br i1 %c, label %y, label %n\ny:\n ret i1 true\nn:\n ret i1 false"
        )
    };
    assert_eq!(verify("", &program(8, 8), 16, 1000), "proved");
    // nuw over an undefined payload byte may be poison.
    assert_eq!(verify("", &program(16, 8), 16, 1000), "counterexample");
    // The payload byte itself is undefined; branching on it is UB.
    assert_eq!(verify("", &program(8, 16), 16, 1000), "counterexample");
}

/// Two masked undef comparisons combined: still any Boolean, absorbed by
/// `and false`, observable otherwise.
#[test]
fn two_undef_booleans_combine_lazily() {
    let program = |mask: &str| {
        format!(
            "start:\n %u = icmp eq i16 undef, 0\n %v = icmp ne i16 undef, 7\n %w = and i1 %u, %v\n %r = and i1 {mask}, %w\n %ok = icmp eq i1 %r, false\n ret i1 %ok"
        )
    };
    assert_eq!(verify("", &program("false"), 16, 1000), "proved");
    // Unmasked, the undef Boolean is never proved (Phage reports it unsupported).
    assert_ne!(verify("", &program("true"), 16, 1000), "proved");
}

#[test]
fn racers_decide_what_the_incremental_solver_cannot() {
    // (w * h) / w == h without overflow: incremental bit-blasting does not
    // finish, cvc5 int-blasting proves it. Without racers it stays unknown.
    let identity = |expected: &str| {
        ir::function(&format!("define i1 @phage_target(i32 %w, i32 %h) {{\nstart:\n %z = icmp eq i32 %w, 0\n br i1 %z, label %ok, label %go\ngo:\n %m = call {{ i32, i1 }} @llvm.umul.with.overflow.i32(i32 %w, i32 %h)\n %o = extractvalue {{ i32, i1 }} %m, 1\n br i1 %o, label %ok, label %div\ndiv:\n %p = extractvalue {{ i32, i1 }} %m, 0\n %q = udiv i32 %p, %w\n %e = icmp eq i32 %q, {expected}\n ret i1 %e\nok:\n ret i1 true\n}}\n"), "phage_target").unwrap()
    };
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    assert_eq!(engine.verify(&identity("%h")).unwrap().status, "proved");
    assert!(engine.solver.escalations_decided > 0);
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    assert_eq!(
        engine.verify(&identity("7")).unwrap().status,
        "counterexample"
    );
    let mut engine = Engine::new(1500, 16, 1000).unwrap();
    engine.solver.portfolio = false;
    assert_eq!(engine.verify(&identity("%h")).unwrap().status, "unknown");
}

/// Undef is any value at each observation, not poison: masked away it is
/// harmless, but branching on it is undefined behavior, a store leaves
/// undefined bytes, and a `noundef` argument rejects it.
#[test]
fn observed_undef_is_any_value_not_poison() {
    let phi = "start:\n br label %m\nm:\n %u = phi i8 [ undef, %start ]\n";
    expect(
        "",
        &format!("{phi} %z = and i8 %u, 0\n %v = icmp eq i8 %z, 0\n ret i1 %v"),
        "proved",
    );
    expect(
        "",
        &format!("{phi} %v = icmp ule i8 %u, 200\n ret i1 %v"),
        "counterexample",
    );
    // Branching on undef: UB even though both successors return true.
    let branch = "start:\n br label %m\nm:\n %u = phi i1 [ undef, %start ]\n br i1 %u, label %a, label %b\na:\n ret i1 true\nb:\n ret i1 true";
    expect("", branch, "counterexample");
    // Stored undef stays undefined bytes: a masked load is fine, a
    // `!noundef` load is UB.
    let stored = format!("{phi} %p = alloca i8, align 1\n store i8 %u, ptr %p, align 1\n");
    expect(
        "",
        &format!(
            "{stored} %l = load i8, ptr %p, align 1\n %z = and i8 %l, 0\n %v = icmp eq i8 %z, 0\n ret i1 %v"
        ),
        "proved",
    );
    expect(
        "",
        &format!("{stored} %l = load i8, ptr %p, align 1, !noundef !0\n ret i1 true"),
        "counterexample",
    );
    // The property itself returning undef may return false.
    expect(
        "",
        "start:\n br label %m\nm:\n %u = phi i1 [ undef, %start ]\n ret i1 %u",
        "counterexample",
    );
}

/// `select` with an undef arm: undef only where that arm is chosen.
#[test]
fn select_keeps_undef_to_its_arm() {
    let select = |c: &str, tail: &str| {
        format!(
            "start:\n br label %m\nm:\n %u = phi i8 [ undef, %start ]\n %r = select i1 {c}, i8 %u, i8 7\n{tail}"
        )
    };
    // Never chosen: exactly 7.
    expect(
        "",
        &select("false", " %v = icmp eq i8 %r, 7\n ret i1 %v"),
        "proved",
    );
    // Chosen: any value at each use, but still defined.
    expect(
        "i1 %c",
        &select(
            "%c",
            " %z = and i8 %r, 0\n %v = icmp eq i8 %z, 0\n ret i1 %v",
        ),
        "proved",
    );
    expect(
        "i1 %c",
        &select("%c", " %v = icmp eq i8 %r, 7\n ret i1 %v"),
        "counterexample",
    );
    // Branching on it where it may be undef is UB.
    let branch = "start:\n br label %m\nm:\n %u = phi i1 [ undef, %start ]\n %r = select i1 %c, i1 %u, i1 true\n br i1 %r, label %a, label %a\na:\n ret i1 true";
    expect("i1 %c", branch, "counterexample");
    expect(
        "i1 %c",
        &branch.replace("i1 %c, i1 %u, i1 true", "i1 false, i1 %u, i1 true"),
        "proved",
    );
}

/// Undef condition with equal arms has no choice to make; an atomic
/// exchange keeps the stored operand's per-byte definedness.
#[test]
fn undef_precision_in_select_and_exchange() {
    let phi = "start:\n br label %m\nm:\n %u = phi i1 [ undef, %start ]\n";
    expect(
        "",
        &format!("{phi} %r = select i1 %u, i1 true, i1 true\n ret i1 %r"),
        "proved",
    );
    expect(
        "i1 %c",
        &format!("{phi} %r = select i1 %u, i1 true, i1 %c\n ret i1 %r"),
        "counterexample",
    );
    let xchg = |tail: &str| {
        format!(
            "start:\n br label %m\nm:\n %u = phi i8 [ undef, %start ]\n %p = alloca i16, align 2\n store i16 0, ptr %p, align 2\n %n = zext i8 %u to i16\n %old = atomicrmw xchg ptr %p, i16 %n monotonic, align 2\n %v = load i16, ptr %p, align 2\n{tail}"
        )
    };
    expect(
        "",
        &xchg(
            " %h = lshr i16 %v, 8\n %b = trunc i16 %h to i8\n %ok = icmp eq i8 %b, 0\n ret i1 %ok",
        ),
        "proved",
    );
    expect(
        "",
        &xchg(" %b = trunc i16 %v to i8\n %ok = icmp eq i8 %b, 0\n ret i1 %ok"),
        "counterexample",
    );
}
