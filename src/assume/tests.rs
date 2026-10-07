//! Real Z3 controls: false/poison guarantees fail and unknown bundles stop.
use crate::{engine::Engine, ir::Module};
use std::rc::Rc;
fn check(declarations: &str, body: &str) -> String {
    let text = format!(
        "{declarations}\ndeclare void @llvm.assume(i1)\ndefine i1 @phage_target() {{\nstart:\n %a = alloca i8, align 1\n{body}\n ret i1 true\n}}\n"
    );
    let module = Module::new("user", text, false).unwrap();
    let entry = module.function("phage_target").unwrap();
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    engine.modules.push(Rc::new(module));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
#[test]
fn nonnull_bundles_are_obligations_with_negative_controls() {
    assert_eq!(
        check("", " call void @llvm.assume(i1 true) [\"nonnull\"(ptr %a)]"),
        "proved"
    );
    assert_eq!(
        check(
            "",
            " call void @llvm.assume(i1 true) [\"nonnull\"(ptr %a), \"nonnull\"(ptr %a)]"
        ),
        "proved"
    );
    for argument in ["null", "poison", "undef"] {
        assert_eq!(
            check(
                "",
                &format!(" call void @llvm.assume(i1 true) [\"nonnull\"(ptr {argument})]")
            ),
            "counterexample"
        );
    }
    assert_eq!(
        check("", " call void @llvm.assume(i1 false)"),
        "counterexample"
    );
}
#[test]
fn other_bundles_and_bad_signatures_remain_unknown() {
    for body in [
        " call void @llvm.assume(i1 true) [\"align\"(ptr %a, i64 8)]",
        " call void @llvm.assume(i1 true) [\"nonnull\"(i64 1)]",
        " call void @llvm.assume(i1 true) [\"nonnull\"(ptr %a, ptr %a)]",
        " call void @llvm.assume(i1 true) [\"nonnull\"(ptr %a) garbage]",
        " call void @llvm.assume(i1 true) [\"future\"(ptr %a)]",
        " call void @llvm.assume(i1 false) [\"nonnull\"(ptr %a)]",
        " call void @other() [\"nonnull\"(ptr %a)]",
        " call void @llvm.assume()",
    ] {
        assert_eq!(check("declare void @other()", body), "unknown", "{body}");
    }
}
#[test]
fn weak_external_address_cannot_silently_become_nonnull() {
    assert_eq!(
        check(
            "declare extern_weak void @optional()",
            " call void @llvm.assume(i1 true) [\"nonnull\"(ptr @optional)]"
        ),
        "unknown"
    );
    assert_eq!(
        check(
            "declare void @present()",
            " call void @llvm.assume(i1 true) [\"nonnull\"(ptr @present)]"
        ),
        "proved"
    );
}
