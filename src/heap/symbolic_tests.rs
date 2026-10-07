//! Exact dynamic-length controls independent of rustc: accesses at n fail,
//! valid accesses prove, layout limits and metadata mismatches do not pass.
use crate::{engine::Engine, ir::Module};
use std::rc::Rc;
fn check(body: &str) -> String {
    let text = format!(
        "declare ptr @__rust_alloc(i64, i64)\ndeclare ptr @__rust_alloc_zeroed(i64, i64)\ndeclare ptr @__rust_realloc(ptr, i64, i64, i64)\ndeclare void @__rust_dealloc(ptr, i64, i64)\ndefine i1 @phage_target(i8 %raw) {{\nentry:\n %small = icmp ule i8 %raw, 64\n %positive = icmp ugt i8 %raw, 0\n %domain = and i1 %small, %positive\n br i1 %domain, label %test, label %excluded\nexcluded:\n ret i1 true\ntest:\n %n = zext i8 %raw to i64\n{body}\n}}\n"
    );
    let module = Module::new("user", text, false).unwrap();
    let entry = module.function("phage_target").unwrap();
    let mut engine = Engine::new(15000, 96, 10000).unwrap();
    engine.modules.push(Rc::new(module));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
#[test]
fn bounds_use_the_exact_requested_length_not_its_upper_bound() {
    assert_eq!(
        check(
            " %p = call ptr @__rust_alloc_zeroed(i64 %n, i64 1)\n %last = sub i64 %n, 1\n %q = getelementptr inbounds i8, ptr %p, i64 %last\n %v = load i8, ptr %q\n %ok = icmp eq i8 %v, 0\n call void @__rust_dealloc(ptr %p, i64 %n, i64 1)\n ret i1 %ok"
        ),
        "proved"
    );
    assert_eq!(
        check(
            " %p = call ptr @__rust_alloc(i64 %n, i64 1)\n %q = getelementptr inbounds i8, ptr %p, i64 %n\n store i8 7, ptr %q\n ret i1 true"
        ),
        "counterexample"
    );
    assert_eq!(
        check(
            " %p = call ptr @__rust_alloc(i64 %n, i64 1)\n %wrong = add i64 %n, 1\n call void @__rust_dealloc(ptr %p, i64 %wrong, i64 1)\n ret i1 true"
        ),
        "counterexample"
    );
}
#[test]
fn symbolic_reallocation_preserves_only_the_exact_common_prefix() {
    let body = " %p = call ptr @__rust_alloc(i64 %n, i64 1)\n store i8 7, ptr %p\n %new = add i64 %n, 8\n %q = call ptr @__rust_realloc(ptr %p, i64 %n, i64 1, i64 %new)\n %v = load i8, ptr %q\n %ok = icmp eq i8 %v, 7\n call void @__rust_dealloc(ptr %q, i64 %new, i64 1)\n ret i1 %ok";
    assert_eq!(check(body), "proved");
    let tail = body.replace(
        "%v = load i8, ptr %q",
        "%tail = getelementptr i8, ptr %q, i64 %n\n %v = load i8, ptr %tail",
    );
    assert_eq!(check(&tail), "counterexample");
}
#[test]
fn invalid_layout_is_a_counterexample() {
    assert_eq!(
        check(" %p = call ptr @__rust_alloc(i64 9223372036854775807, i64 8)\n ret i1 true"),
        "counterexample"
    );
    assert_eq!(
        check(" %p = call ptr @__rust_alloc(i64 %n, i64 3)\n ret i1 true"),
        "counterexample"
    );
}

#[test]
fn pointer_integer_copies_preserve_only_the_explicit_origin() {
    let body = " %p = call ptr @__rust_alloc(i64 %n, i64 1)\n %slot = alloca i64, align 8\n %bits = ptrtoint ptr %p to i64\n store i64 %bits, ptr %slot, align 8\n %copy = load ptr, ptr %slot, align 8\n store i8 9, ptr %copy\n %v = load i8, ptr %p\n %ok = icmp eq i8 %v, 9\n call void @__rust_dealloc(ptr %p, i64 %n, i64 1)\n ret i1 %ok";
    assert_eq!(check(body), "proved");
    let corrupt = body.replace(
        "store i64 %bits, ptr %slot",
        "%altered = xor i64 %bits, 1\n store i64 %altered, ptr %slot",
    );
    assert_eq!(check(&corrupt), "unknown");
}

#[test]
fn unbounded_symbolic_extent_is_a_specific_unknown_not_an_assumed_cap() {
    let text = "declare ptr @__rust_alloc(i64, i64)\ndefine i1 @phage_target(i32 %raw) {\nentry:\n %n = zext i32 %raw to i64\n %zero = icmp eq i64 %n, 0\n br i1 %zero, label %empty, label %allocate\nempty:\n ret i1 true\nallocate:\n %p = call ptr @__rust_alloc(i64 %n, i64 1)\n ret i1 true\n}\n";
    let module = Module::new("user", text.into(), false).unwrap();
    let entry = module.function("phage_target").unwrap();
    let mut engine = Engine::new(15000, 32, 1000).unwrap();
    engine.modules.push(Rc::new(module));
    let verdict = engine.verify(&entry).unwrap();
    assert_eq!(verdict.status, "unknown");
    assert!(
        verdict
            .detail
            .contains("symbolic heap length exceeds the 1048576-byte"),
        "{}",
        verdict.detail
    );
}
