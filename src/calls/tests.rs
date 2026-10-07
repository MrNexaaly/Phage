//! Controls for call contracts: result attributes (`range`, `nonnull`,
//! `align`) on call sites and definitions, attribute poisoning of byte
//! intrinsic arguments, and metadata parsing. Real Z3 is required.

use crate::{engine::Engine, ir::Module};
use std::rc::Rc;

fn verify(text: &str) -> String {
    let module = Module::new("user", text.to_owned(), false).expect("fixture parses");
    let entry = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}

#[test]
fn violated_result_attributes_make_the_result_poison() {
    let seven = "define i8 @seven() {\nstart:\n ret i8 7\n}\n";
    let call = |site: &str| {
        format!(
            "{seven}define i1 @phage_target() {{\nstart:\n %x = call {site} i8 @seven()\n %ok = icmp eq i8 %x, 7\n ret i1 %ok\n}}\n"
        )
    };
    assert_eq!(verify(&call("range(i8 0, 8)")), "proved");
    assert_eq!(verify(&call("range(i8 0, 5)")), "counterexample");
    assert_eq!(verify(&call("noundef range(i8 0, 5)")), "counterexample");
    // The definition's own return attributes count as well.
    let declared = "define range(i8 0, 5) i8 @g() {\nstart:\n ret i8 7\n}\ndefine i1 @phage_target() {\nstart:\n %x = call i8 @g()\n %ok = icmp eq i8 %x, 7\n ret i1 %ok\n}\n";
    assert_eq!(verify(declared), "counterexample");
    // A modeled call: cttz of a nonzero i16 is at most 15.
    let modeled = |bounds: &str| {
        format!(
            "declare i16 @llvm.cttz.i16(i16, i1)\ndefine i1 @phage_target(i16 %x) {{\nstart:\n %y = or i16 %x, 1\n %c = call range(i16 {bounds}) i16 @llvm.cttz.i16(i16 %y, i1 true)\n %ok = icmp eq i16 %c, %c\n ret i1 %ok\n}}\n"
        )
    };
    assert_eq!(verify(&modeled("0, 17")), "proved");
    assert_eq!(verify(&modeled("1, 17")), "counterexample");
}

#[test]
fn byte_intrinsics_are_exempt_only_for_alignment() {
    let fill = |length: &str| {
        format!(
            "declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)\ndefine i1 @phage_target() {{\nstart:\n %p = alloca i8, align 1\n call void @llvm.memset.p0.i64(ptr %p, i8 0, i64 {length}, i1 false)\n ret i1 true\n}}\n"
        )
    };
    assert_eq!(verify(&fill("1")), "proved");
    // Violated inferred attributes are still part of the LLVM artifact.
    // Do not silently check the unannotated fill instead.
    assert_eq!(verify(&fill("range(i64 2, 3) 1")), "unknown");
    // The fill's own obligations still hold: two bytes into one is invalid.
    assert_eq!(verify(&fill("range(i64 2, 3) 2")), "counterexample");
}

#[test]
fn intrinsic_argument_ranges_respect_boundaries_and_path_constraints() {
    let call = |parameter: &str, prefix: &str, argument: &str, bounds: &str| {
        format!(
            "declare i8 @llvm.ctpop.i8(i8)\ndefine i1 @phage_target({parameter}) {{\nstart:\n {prefix}\n %n = call i8 @llvm.ctpop.i8(i8 range(i8 {bounds}) {argument})\n %ok = icmp eq i8 %n, %n\n ret i1 %ok\n}}\n"
        )
    };
    for (bounds, argument, expected) in [
        ("1, 2", "0", "unknown"),
        ("1, 2", "1", "proved"),
        ("1, 2", "2", "unknown"),
        ("-2, 2", "-1", "proved"),
        ("-2, 2", "2", "unknown"),
    ] {
        assert_eq!(verify(&call("", "", argument, bounds)), expected);
    }
    assert_eq!(verify(&call("i8 %x", "", "%x", "1, 2")), "unknown");
    // Prove the contract from path facts; never simply assume it.
    let guarded = " %valid = icmp eq i8 %x, 1\n br i1 %valid, label %check, label %skip\nskip:\n ret i1 true\ncheck:";
    assert_eq!(verify(&call("i8 %x", guarded, "%x", "1, 2")), "proved");
    // An unreachable violating call does not block unrelated valid proofs.
    let unreachable = " br i1 false, label %check, label %skip\nskip:\n ret i1 true\ncheck:";
    assert_eq!(verify(&call("", unreachable, "0", "1, 2")), "proved");
}

#[test]
fn metadata_is_found_however_it_is_spaced() {
    let load = |attachment: &str| {
        format!(
            "define i1 @phage_target() {{\nstart:\n %p = alloca i8, align 1\n %x = load i8, ptr %p, align 1{attachment}\n ret i1 true\n}}\n"
        )
    };
    assert_eq!(verify(&load(", !noundef !0")), "counterexample");
    assert_eq!(verify(&load(",!noundef !0")), "counterexample");
    assert_eq!(verify(&load(", !dbg !3, !noundef !0")), "counterexample");
    assert_eq!(verify(&load(", !noundefx !0")), "proved");
    assert_eq!(verify(&load("")), "proved");
}

#[test]
fn vector_access_alignment_defaults_to_its_size() {
    let store = |offset: u32| {
        format!(
            "define i1 @phage_target() {{\nstart:\n %p = alloca [256 x i8], align 128\n %q = getelementptr i8, ptr %p, i64 {offset}\n store <16 x i64> zeroinitializer, ptr %q\n ret i1 true\n}}\n"
        )
    };
    assert_eq!(verify(&store(128)), "proved");
    assert_eq!(verify(&store(64)), "counterexample");
}
