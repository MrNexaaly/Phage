//! Floating-point controls, each with a failing or unknown twin. Expected
//! verdicts were checked against native Rust where a Rust form exists
//! (`(x * 0.1) * 10.0 == x` fails for x = 13; halving and doubling is exact).

use crate::{engine::Engine, ir::Module};
use std::rc::Rc;

fn verify(parameters: &str, body: &str) -> String {
    let text = format!(
        "declare i8 @llvm.fptoui.sat.i8.f32(float)\ndeclare float @llvm.fabs.f32(float)\ndeclare float @llvm.copysign.f32(float, float)\ndeclare float @llvm.floor.f32(float)\ndeclare double @llvm.sqrt.f64(double)\ndeclare i1 @llvm.is.fpclass.f32(float, i32)\ndeclare float @llvm.minnum.f32(float, float)\ndeclare float @llvm.sqrt.f32(float)\ndefine i1 @phage_target({parameters}) {{\nstart:\n{body}\n}}\n"
    );
    let module = Module::new("user", text, false).expect("parses");
    let entry = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}

#[test]
fn rounding_matches_ieee_single_precision() {
    // Halving and doubling a byte is exact; a tenth times ten is not.
    let halve = " %f = uitofp i8 %x to float\n %h = fmul float %f, 5.000000e-01\n %d = fmul float %h, 2.000000e+00\n %y = call i8 @llvm.fptoui.sat.i8.f32(float %d)\n %v = icmp eq i8 %y, %x\n ret i1 %v";
    assert_eq!(verify("i8 %x", halve), "proved");
    let tenth = " %f = uitofp i8 %x to float\n %t = fmul float %f, 0x3FB99999A0000000\n %d = fmul float %t, 1.000000e+01\n %v = fcmp oeq float %d, %f\n ret i1 %v";
    assert_eq!(verify("i8 %x", tenth), "counterexample");
}

#[test]
fn nan_is_unordered_and_its_bits_are_any_nan() {
    let nan = " %f = uitofp i8 %x to float\n %z = fsub float %f, %f\n %n = fdiv float %z, %z\n";
    assert_eq!(
        verify(
            "i8 %x",
            &format!("{nan} %v = fcmp oeq float %n, %n\n ret i1 %v")
        ),
        "counterexample"
    );
    assert_eq!(
        verify(
            "i8 %x",
            &format!("{nan} %v = fcmp uno float %n, %n\n ret i1 %v")
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "i8 %x",
            &format!("{nan} %v = fcmp une float %n, %n\n ret i1 %v")
        ),
        "proved"
    );
    // A NaN's payload is not fixed: one particular pattern is refuted...
    assert_eq!(
        verify(
            "i8 %x",
            &format!(
                "{nan} %b = bitcast float %n to i32\n %v = icmp eq i32 %b, 2143289344\n ret i1 %v"
            )
        ),
        "counterexample"
    );
    // ...but every pattern has an all-ones exponent and a nonzero fraction.
    assert_eq!(
        verify(
            "i8 %x",
            &format!(
                "{nan} %b = bitcast float %n to i32\n %e = and i32 %b, 2139095040\n %m = and i32 %b, 8388607\n %e1 = icmp eq i32 %e, 2139095040\n %m1 = icmp ne i32 %m, 0\n %v = and i1 %e1, %m1\n ret i1 %v"
            )
        ),
        "proved"
    );
}

#[test]
fn signed_zero_and_bits_round_trip() {
    let neg = " %z = fneg float 0.000000e+00\n %b = bitcast float %z to i32\n";
    assert_eq!(
        verify(
            "",
            &format!("{neg} %v = icmp eq i32 %b, -2147483648\n ret i1 %v")
        ),
        "proved"
    );
    assert_eq!(
        verify("", &format!("{neg} %v = icmp eq i32 %b, 0\n ret i1 %v")),
        "counterexample"
    );
    // Bitcast, store and load preserve bits exactly, NaN payloads too...
    let round = " %f = bitcast i32 %x to float\n %p = alloca float, align 4\n store float %f, ptr %p, align 4\n %l = load i32, ptr %p, align 4\n";
    assert_eq!(
        verify(
            "i32 %x",
            &format!("{round} %v = icmp eq i32 %l, %x\n ret i1 %v")
        ),
        "proved"
    );
    // ...while arithmetic may change a NaN's payload (x + -0.0 is x otherwise).
    let add = " %f = bitcast i32 %x to float\n %s = fadd float %f, -0.000000e+00\n %b = bitcast float %s to i32\n %v = icmp eq i32 %b, %x\n ret i1 %v";
    assert_eq!(verify("i32 %x", add), "counterexample");
    let ordered = " %f = bitcast i32 %x to float\n %o = fcmp ord float %f, %f\n br i1 %o, label %num, label %nan\nnum:\n %s = fadd float %f, -0.000000e+00\n %b = bitcast float %s to i32\n %v = icmp eq i32 %b, %x\n ret i1 %v\nnan:\n ret i1 true";
    assert_eq!(verify("i32 %x", ordered), "proved");
    let finite = " %f = uitofp i8 %x to float\n %p = alloca float, align 4\n store float %f, ptr %p, align 4\n %l = load float, ptr %p, align 4\n %v = fcmp oeq float %l, %f\n ret i1 %v";
    assert_eq!(verify("i8 %x", finite), "proved");
}

#[test]
fn conversions_out_of_range_are_poison_or_saturate() {
    // 300.0 does not fit i8: plain fptoui is poison, the .sat form clamps.
    assert_eq!(
        verify(
            "",
            " %y = fptoui float 3.000000e+02 to i8\n %v = icmp eq i8 %y, %y\n ret i1 %v"
        ),
        "counterexample"
    );
    assert_eq!(
        verify(
            "",
            " %y = call i8 @llvm.fptoui.sat.i8.f32(float 3.000000e+02)\n %v = icmp eq i8 %y, -1\n ret i1 %v"
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "",
            " %y = fptosi float -1.250000e+00 to i8\n %v = icmp eq i8 %y, -1\n ret i1 %v"
        ),
        "proved"
    );
    // Widening then narrowing a float is exact.
    let wide = " %f = sitofp i16 %x to float\n %d = fpext float %f to double\n %g = fptrunc double %d to float\n %v = fcmp oeq float %g, %f\n ret i1 %v";
    assert_eq!(verify("i16 %x", wide), "proved");
}

#[test]
fn intrinsics_follow_libm_semantics() {
    assert_eq!(
        verify(
            "",
            " %r = call double @llvm.sqrt.f64(double 4.000000e+00)\n %v = fcmp oeq double %r, 2.000000e+00\n ret i1 %v"
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "",
            " %r = call double @llvm.sqrt.f64(double 2.000000e+00)\n %s = fmul double %r, %r\n %v = fcmp oeq double %s, 2.000000e+00\n ret i1 %v"
        ),
        "counterexample"
    );
    assert_eq!(
        verify(
            "",
            " %r = call float @llvm.floor.f32(float -1.500000e+00)\n %v = fcmp oeq float %r, -2.000000e+00\n ret i1 %v"
        ),
        "proved"
    );
    let sign = " %f = uitofp i8 %x to float\n %c = call float @llvm.copysign.f32(float %f, float -1.000000e+00)\n %a = call float @llvm.fabs.f32(float %c)\n";
    assert_eq!(
        verify(
            "i8 %x",
            &format!("{sign} %v = fcmp oeq float %a, %f\n ret i1 %v")
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "i8 %x",
            &format!("{sign} %v = fcmp ogt float %c, 0.000000e+00\n ret i1 %v")
        ),
        "counterexample"
    );
}

#[test]
fn unmodeled_float_semantics_are_unknown() {
    assert_eq!(
        verify(
            "",
            " %r = frem float 5.000000e+00, 3.000000e+00\n ret i1 true"
        ),
        "unknown"
    );
    assert_eq!(
        verify(
            "",
            " %r = fadd fast float 1.000000e+00, 2.000000e+00\n ret i1 true"
        ),
        "unknown"
    );
}

/// `llvm.is.fpclass` reads LangRef's classes from the bits.
#[test]
fn fpclass_reads_the_bits() {
    // -0.0 is class 5 (negative zero), not 6; a quiet NaN is class 1.
    let neg = " %z = fneg float 0.000000e+00\n";
    assert_eq!(
        verify(
            "",
            &format!("{neg} %c = call i1 @llvm.is.fpclass.f32(float %z, i32 32)\n ret i1 %c")
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "",
            &format!("{neg} %c = call i1 @llvm.is.fpclass.f32(float %z, i32 64)\n ret i1 %c")
        ),
        "counterexample"
    );
    let nan = " %n = bitcast i32 2143289344 to float\n";
    assert_eq!(
        verify(
            "",
            &format!("{nan} %c = call i1 @llvm.is.fpclass.f32(float %n, i32 2)\n ret i1 %c")
        ),
        "proved"
    );
    // Any input is in exactly one class: all ten bits cover it, nine do not.
    assert_eq!(
        verify(
            "i32 %x",
            " %f = bitcast i32 %x to float\n %c = call i1 @llvm.is.fpclass.f32(float %f, i32 1023)\n ret i1 %c"
        ),
        "proved"
    );
    assert_eq!(
        verify(
            "i32 %x",
            " %f = bitcast i32 %x to float\n %c = call i1 @llvm.is.fpclass.f32(float %f, i32 1022)\n ret i1 %c"
        ),
        "counterexample"
    );
}

/// Review regressions: computed NaNs are quiet; opposite zeros in minnum
/// are chosen per call; call fast-math flags poison; alloca alignment.
#[test]
fn review_regressions() {
    let quiet = " %n = bitcast i32 2143289344 to float\n %r = fadd float %n, 0.000000e+00\n %c = call i1 @llvm.is.fpclass.f32(float %r, i32 1)\n %v = xor i1 %c, true\n ret i1 %v";
    assert_eq!(verify("", quiet), "proved");
    let zeros = " %z = fneg float 0.000000e+00\n %a = call float @llvm.minnum.f32(float 0.000000e+00, float %z)\n %b = call float @llvm.minnum.f32(float 0.000000e+00, float %z)\n %x = bitcast float %a to i32\n %y = bitcast float %b to i32\n %v = icmp eq i32 %x, %y\n ret i1 %v";
    assert_eq!(verify("", zeros), "counterexample");
    let flagged = " %r = call nnan float @llvm.sqrt.f32(float -1.000000e+00)\n %v = fcmp uno float %r, %r\n ret i1 %v";
    assert_eq!(verify("", flagged), "counterexample");
    assert_eq!(verify("", &flagged.replace("call nnan", "call")), "proved");
    assert_eq!(
        verify("", &flagged.replace("call nnan", "call fast")),
        "unknown"
    );
    let implicit = " %p = alloca float\n store float 1.000000e+00, ptr %p\n %x = load float, ptr %p\n %v = fcmp oeq float %x, 1.000000e+00\n ret i1 %v";
    assert_eq!(verify("", implicit), "proved");
}
