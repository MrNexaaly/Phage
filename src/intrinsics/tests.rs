//! Positive, negative and unknown controls for integer intrinsics, switch,
//! assumptions and diverging declarations. Real Z3 is required.

use crate::{engine::Engine, ir};

fn verify_module(module: &str) -> String {
    let function = ir::function(module, "phage_target").expect("fixture must parse");
    Engine::new(5000, 16, 1000)
        .expect("Z3 required")
        .verify(&function)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
fn expect(parameters: &str, blocks: &str, status: &str) {
    let module = format!("define i1 @phage_target({parameters}) {{\n{blocks}\n}}\n");
    assert_eq!(verify_module(&module), status, "{blocks}");
}
/// `%r = <call>` compared against `expected`, for every input.
fn equals(parameters: &str, call: &str, ty: &str, expected: &str, status: &str) {
    expect(
        parameters,
        &format!("start:\n %r = call {ty} {call}\n %v = icmp eq {ty} %r, {expected}\n ret i1 %v"),
        status,
    );
}

#[test]
fn min_max_select_the_right_operand() {
    for (op, cmp) in [
        ("umin", "ule"),
        ("umax", "uge"),
        ("smin", "sle"),
        ("smax", "sge"),
    ] {
        expect(
            "i8 %x, i8 %y",
            &format!(
                "start:\n %r = call i8 @llvm.{op}.i8(i8 %x, i8 %y)\n %a = icmp {cmp} i8 %r, %x\n %b = icmp {cmp} i8 %r, %y\n %v = and i1 %a, %b\n ret i1 %v"
            ),
            "proved",
        );
        equals(
            "i8 %x, i8 %y",
            &format!("@llvm.{op}.i8(i8 %x, i8 %y)"),
            "i8",
            "%x",
            "counterexample",
        );
    }
    // Signed and unsigned orders differ at the sign bit.
    equals("", "@llvm.smin.i8(i8 -1, i8 1)", "i8", "-1", "proved");
    equals("", "@llvm.umin.i8(i8 -1, i8 1)", "i8", "1", "proved");
}
#[test]
fn abs_honors_int_min_poison_flag() {
    equals("", "@llvm.abs.i8(i8 -5, i1 false)", "i8", "5", "proved");
    equals(
        "",
        "@llvm.abs.i8(i8 -128, i1 false)",
        "i8",
        "-128",
        "proved",
    );
    // Returning a poisoned comparison is observable.
    equals(
        "",
        "@llvm.abs.i8(i8 -128, i1 true)",
        "i8",
        "-128",
        "counterexample",
    );
    equals(
        "i8 %x",
        "@llvm.abs.i8(i8 %x, i1 false)",
        "i8",
        "%x",
        "counterexample",
    );
}
#[test]
fn byte_and_bit_reversal() {
    equals(
        "",
        "@llvm.bswap.i32(i32 287454020)",
        "i32",
        "1144201745",
        "proved",
    );
    expect(
        "i32 %x",
        "start:\n %a = call i32 @llvm.bswap.i32(i32 %x)\n %b = call i32 @llvm.bswap.i32(i32 %a)\n %v = icmp eq i32 %b, %x\n ret i1 %v",
        "proved",
    );
    equals(
        "i16 %x",
        "@llvm.bswap.i16(i16 %x)",
        "i16",
        "%x",
        "counterexample",
    );
    equals("", "@llvm.bitreverse.i8(i8 1)", "i8", "-128", "proved");
    equals("", "@llvm.bitreverse.i8(i8 6)", "i8", "96", "proved");
}
#[test]
fn population_and_zero_counts() {
    equals("", "@llvm.ctpop.i8(i8 -1)", "i8", "8", "proved");
    equals("", "@llvm.ctpop.i16(i16 4097)", "i16", "2", "proved");
    expect(
        "i8 %x",
        "start:\n %r = call i8 @llvm.ctpop.i8(i8 %x)\n %v = icmp ule i8 %r, 8\n ret i1 %v",
        "proved",
    );
    equals(
        "i8 %x",
        "@llvm.ctpop.i8(i8 %x)",
        "i8",
        "1",
        "counterexample",
    );
    equals("", "@llvm.ctlz.i8(i8 1, i1 false)", "i8", "7", "proved");
    equals("", "@llvm.ctlz.i8(i8 0, i1 false)", "i8", "8", "proved");
    equals("", "@llvm.cttz.i8(i8 8, i1 false)", "i8", "3", "proved");
    equals("", "@llvm.cttz.i32(i32 0, i1 false)", "i32", "32", "proved");
    // is_zero_poison makes a zero input poison.
    equals(
        "",
        "@llvm.ctlz.i8(i8 0, i1 true)",
        "i8",
        "8",
        "counterexample",
    );
    equals(
        "",
        "@llvm.cttz.i8(i8 0, i1 true)",
        "i8",
        "8",
        "counterexample",
    );
}
#[test]
fn funnel_shifts_are_rotations_of_the_concatenation() {
    equals(
        "",
        "@llvm.fshl.i8(i8 -127, i8 -127, i8 1)",
        "i8",
        "3",
        "proved",
    );
    equals(
        "",
        "@llvm.fshr.i8(i8 3, i8 3, i8 1)",
        "i8",
        "-127",
        "proved",
    );
    // The shift amount is taken modulo the width.
    equals("", "@llvm.fshl.i8(i8 1, i8 0, i8 9)", "i8", "2", "proved");
    equals(
        "i8 %x",
        "@llvm.fshl.i8(i8 %x, i8 %x, i8 8)",
        "i8",
        "%x",
        "proved",
    );
    expect(
        "i8 %x",
        "start:\n %a = call i8 @llvm.fshl.i8(i8 %x, i8 %x, i8 3)\n %b = call i8 @llvm.fshr.i8(i8 %x, i8 %x, i8 5)\n %v = icmp eq i8 %a, %b\n ret i1 %v",
        "proved",
    );
    equals(
        "i8 %x",
        "@llvm.fshl.i8(i8 %x, i8 %x, i8 3)",
        "i8",
        "%x",
        "counterexample",
    );
}
#[test]
fn saturating_arithmetic_clamps() {
    equals("", "@llvm.uadd.sat.i8(i8 -6, i8 10)", "i8", "-1", "proved");
    equals("", "@llvm.usub.sat.i8(i8 3, i8 5)", "i8", "0", "proved");
    equals(
        "",
        "@llvm.sadd.sat.i8(i8 100, i8 100)",
        "i8",
        "127",
        "proved",
    );
    equals(
        "",
        "@llvm.sadd.sat.i8(i8 -100, i8 -100)",
        "i8",
        "-128",
        "proved",
    );
    equals(
        "",
        "@llvm.ssub.sat.i8(i8 -100, i8 100)",
        "i8",
        "-128",
        "proved",
    );
    equals(
        "",
        "@llvm.ssub.sat.i8(i8 100, i8 -100)",
        "i8",
        "127",
        "proved",
    );
    equals("", "@llvm.sadd.sat.i8(i8 -3, i8 5)", "i8", "2", "proved");
    expect(
        "i8 %x, i8 %y",
        "start:\n %r = call i8 @llvm.uadd.sat.i8(i8 %x, i8 %y)\n %v = icmp uge i8 %r, %x\n ret i1 %v",
        "proved",
    );
    equals(
        "i8 %x, i8 %y",
        "@llvm.sadd.sat.i8(i8 %x, i8 %y)",
        "i8",
        "%x",
        "counterexample",
    );
}
#[test]
fn three_way_compare_uses_result_width() {
    equals("", "@llvm.scmp.i8.i32(i32 -1, i32 1)", "i8", "-1", "proved");
    equals("", "@llvm.ucmp.i8.i32(i32 -1, i32 1)", "i8", "1", "proved");
    equals(
        "i32 %x",
        "@llvm.scmp.i8.i32(i32 %x, i32 %x)",
        "i8",
        "0",
        "proved",
    );
    equals(
        "i32 %x, i32 %y",
        "@llvm.ucmp.i8.i32(i32 %x, i32 %y)",
        "i8",
        "0",
        "counterexample",
    );
}
#[test]
fn assumptions_are_obligations() {
    // Reachable false assumption: undefined behavior, not a trusted fact.
    expect(
        "i8 %x",
        "start:\n %c = icmp ult i8 %x, 10\n call void @llvm.assume(i1 %c)\n ret i1 true",
        "counterexample",
    );
    // An implied assumption is harmless.
    expect(
        "i8 %x",
        "start:\n %c = icmp ult i8 %x, 10\n br i1 %c, label %a, label %b\na:\n call void @llvm.assume(i1 %c)\n ret i1 true\nb:\n ret i1 true",
        "proved",
    );
    // Operand-bundle assumptions carry unchecked facts, with any spacing;
    // the bundle's last operand must never be read as the condition.
    for bundle in [
        " [ \"align\"(ptr %p, i64 1) ]",
        " [\"align\"(ptr %p, i64 1)]",
    ] {
        for condition in ["true", "false"] {
            expect(
                "",
                &format!(
                    "start:\n %p = alloca i8, align 1\n call void @llvm.assume(i1 {condition}){bundle}\n ret i1 true"
                ),
                "unknown",
            );
        }
    }
}
#[test]
fn multi_line_switch_forks_every_case() {
    let switch = |ret_default: &str| {
        format!(
            "start:\n switch i8 %x, label %other [\n i8 0, label %zero\n i8 -1, label %max\n ]\nzero:\n ret i1 true\nmax:\n %m = icmp eq i8 %x, -1\n ret i1 %m\nother:\n %z = icmp ne i8 %x, 0\n %n = icmp ne i8 %x, -1\n %d = and i1 %z, %n\n %v = and i1 %d, {ret_default}\n ret i1 %v"
        )
    };
    expect("i8 %x", &switch("true"), "proved");
    expect("i8 %x", &switch("false"), "counterexample");
    expect(
        "",
        "start:\n switch i8 poison, label %a [\n i8 0, label %a\n ]\na:\n ret i1 true",
        "counterexample",
    );
}
#[test]
fn diverging_declarations_are_classified() {
    let module = |callee: &str, reachable: &str| {
        format!(
            "define i1 @phage_target(i8 %x) {{\nstart:\n %c = icmp eq i8 %x, {reachable}\n br i1 %c, label %fail, label %ok\nfail:\n call void @{callee}() #3\n unreachable\nok:\n ret i1 true\n}}\n\ndeclare void @{callee}() unnamed_addr #7\n\nattributes #3 = {{ noinline }}\nattributes #7 = {{ cold noinline noreturn nounwind }}\n"
        )
    };
    let unwrap = "_ZN4core6option13unwrap_failed17h0123456789abcdefE";
    let v0 = "_RNvNtCs1234_5alloc7raw_vec17capacity_overflow";
    let exit = "_ZN3std7process4exit17h0123456789abcdefE";
    assert_eq!(verify_module(&module(unwrap, "3")), "counterexample");
    assert_eq!(verify_module(&module(v0, "3")), "counterexample");
    // process::exit never returns either: the property cannot return true.
    assert_eq!(verify_module(&module(exit, "3")), "counterexample");
    // An infeasible guard makes the failure call unreachable.
    let infeasible = module(unwrap, "3").replace("icmp eq i8 %x, 3", "icmp ne i8 %x, %x");
    assert_eq!(verify_module(&infeasible), "proved");
    // Without noreturn the same core symbol is an ordinary unmodeled call.
    let plain = module(unwrap, "3").replace("cold noinline noreturn nounwind", "cold");
    assert_eq!(verify_module(&plain), "unknown");
}
#[test]
fn wide_population_count_stays_tractable() {
    // Setting one bit raises the count by at most one. (The 64-bit form of
    // this property takes seconds; 16 bits keeps the control fast.)
    expect(
        "i16 %x, i16 %y",
        "start:\n %m = and i16 %y, 15\n %b = shl i16 1, %m\n %s = or i16 %x, %b\n %a = call i16 @llvm.ctpop.i16(i16 %x)\n %c = call i16 @llvm.ctpop.i16(i16 %s)\n %d = sub i16 %c, %a\n %v = icmp ule i16 %d, 1\n ret i1 %v",
        "proved",
    );
    expect(
        "i16 %x",
        "start:\n %a = call i16 @llvm.ctpop.i16(i16 %x)\n %v = icmp ule i16 %a, 15\n ret i1 %v",
        "counterexample",
    );
    equals("", "@llvm.ctpop.i64(i64 -1)", "i64", "64", "proved");
    equals("", "@llvm.ctpop.i3(i3 -1)", "i3", "3", "proved");
}
#[test]
fn signed_division_rules() {
    equals("", "@llvm.smin.i8(i8 0, i8 0)", "i8", "0", "proved");
    expect(
        "i8 %x, i8 %y",
        "start:\n %z = icmp eq i8 %y, 0\n br i1 %z, label %ok, label %go\ngo:\n %q = sdiv i8 %x, %y\n %v = icmp sle i8 %q, 127\n ret i1 %v\nok:\n ret i1 true",
        "counterexample",
    );
    expect(
        "i8 %x, i8 %y",
        "start:\n %z = icmp eq i8 %y, 0\n %m = icmp eq i8 %y, -1\n %bad = or i1 %z, %m\n br i1 %bad, label %ok, label %go\ngo:\n %q = sdiv i8 %x, %y\n %r = srem i8 %x, %y\n %p = mul i8 %q, %y\n %s = add i8 %p, %r\n %v = icmp eq i8 %s, %x\n ret i1 %v\nok:\n ret i1 true",
        "proved",
    );
    expect(
        "",
        "start:\n %q = sdiv i8 -7, 2\n %r = srem i8 -7, 2\n %a = icmp eq i8 %q, -3\n %b = icmp eq i8 %r, -1\n %v = and i1 %a, %b\n ret i1 %v",
        "proved",
    );
    expect(
        "",
        "start:\n %q = sdiv exact i8 7, 2\n %v = icmp eq i8 %q, %q\n ret i1 %v",
        "counterexample",
    );
}
