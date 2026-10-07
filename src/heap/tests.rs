//! Controls for calls, the allocator ABI, pointer provenance in memory,
//! byte operations, dangling constants and aggregates. Real Z3 is required.

use crate::{
    engine::{Engine, Verdict},
    ir::Module,
};
use std::rc::Rc;

/// Runs `@phage_target` with every function of `module` callable.
fn run(body: &str) -> Result<Verdict, String> {
    let text = format!(
        "{body}\ndeclare ptr @__rust_alloc(i64, i64)\ndeclare ptr @__rust_alloc_zeroed(i64, i64)\ndeclare void @__rust_dealloc(ptr, i64, i64)\ndeclare ptr @__rust_realloc(ptr, i64, i64, i64)\n"
    );
    let module = Module::new("user", text, false).expect("fixture must parse");
    let entry = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    engine.verify(&entry)
}
fn verify(body: &str) -> String {
    run(body)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
fn target(parameters: &str, blocks: &str) -> String {
    format!("define i1 @phage_target({parameters}) {{\n{blocks}\n}}")
}

#[test]
fn calls_bind_arguments_and_results() {
    let helper = "define i8 @twice(i8 %v) {\nentry:\n %r = shl nuw i8 %v, 1\n ret i8 %r\n}\n";
    let checked = target(
        "i8 %x",
        "start:\n %s = lshr i8 %x, 1\n %r = call i8 @twice(i8 %s)\n %v = icmp ule i8 %r, %x\n ret i1 %v",
    );
    assert_eq!(verify(&format!("{helper}{checked}")), "proved");
    // Overflow poison computed in a callee is observed by the caller's return.
    let unchecked = target(
        "i8 %x",
        "start:\n %r = call i8 @twice(i8 %x)\n %v = icmp uge i8 %r, 0\n ret i1 %v",
    );
    assert_eq!(verify(&format!("{helper}{unchecked}")), "counterexample");
}
#[test]
fn unbounded_recursion_reaches_the_depth_limit() {
    let forever = "define i1 @spin(i8 %x) {\nentry:\n %r = call i1 @spin(i8 %x)\n ret i1 %r\n}\n";
    let entry = target("i8 %x", "start:\n %r = call i1 @spin(i8 %x)\n ret i1 %r");
    assert_eq!(verify(&format!("{forever}{entry}")), "unknown");
}
#[test]
fn void_callee_writes_through_a_pointer() {
    let set =
        "define void @set(ptr %p, i8 %v) {\nentry:\n store i8 %v, ptr %p, align 1\n ret void\n}\n";
    let entry = target(
        "i8 %x",
        "start:\n %p = alloca i8, align 1\n call void @set(ptr %p, i8 %x)\n %l = load i8, ptr %p, align 1\n %v = icmp eq i8 %l, %x\n ret i1 %v",
    );
    assert_eq!(verify(&format!("{set}{entry}")), "proved");
}
#[test]
fn heap_round_trip_and_release() {
    let body = |free: &str| {
        target(
            "i8 %x",
            &format!(
                "start:\n %p = call ptr @__rust_alloc(i64 4, i64 1)\n store i8 %x, ptr %p, align 1\n %l = load i8, ptr %p, align 1\n{free}\n %v = icmp eq i8 %l, %x\n ret i1 %v"
            ),
        )
    };
    let free = " call void @__rust_dealloc(ptr %p, i64 4, i64 1)";
    assert_eq!(verify(&body(free)), "proved");
    let double = format!("{free}\n{free}");
    assert_eq!(verify(&body(&double)), "counterexample");
    assert_eq!(
        verify(&body(" call void @__rust_dealloc(ptr %p, i64 3, i64 1)")),
        "counterexample"
    );
    assert_eq!(
        verify(&body(" call void @__rust_dealloc(ptr %p, i64 4, i64 2)")),
        "counterexample"
    );
    let interior = " %q = getelementptr inbounds i8, ptr %p, i64 1\n call void @__rust_dealloc(ptr %q, i64 4, i64 1)";
    assert_eq!(verify(&body(interior)), "counterexample");
}
#[test]
fn use_after_free_and_stack_free_are_failures() {
    let after = target(
        "i8 %x",
        "start:\n %p = call ptr @__rust_alloc(i64 1, i64 1)\n store i8 %x, ptr %p, align 1\n call void @__rust_dealloc(ptr %p, i64 1, i64 1)\n %l = load i8, ptr %p, align 1\n %v = icmp eq i8 %l, %x\n ret i1 %v",
    );
    assert_eq!(verify(&after), "counterexample");
    let stack = target(
        "",
        "start:\n %p = alloca i8, align 1\n call void @__rust_dealloc(ptr %p, i64 1, i64 1)\n ret i1 true",
    );
    assert_eq!(verify(&stack), "counterexample");
    let zero = target(
        "",
        "start:\n %p = call ptr @__rust_alloc(i64 0, i64 1)\n ret i1 true",
    );
    assert_eq!(verify(&zero), "counterexample");
}
#[test]
fn realloc_keeps_bytes_and_zeroed_memory_is_initialized() {
    let grow = |read: &str| {
        target(
            "i8 %x",
            &format!(
                "start:\n %p = call ptr @__rust_alloc(i64 1, i64 1)\n store i8 %x, ptr %p, align 1\n %q = call ptr @__rust_realloc(ptr %p, i64 1, i64 1, i64 8)\n %a = getelementptr inbounds i8, ptr %q, i64 {read}\n %l = load i8, ptr %a, align 1\n %v = icmp eq i8 %l, %x\n ret i1 %v"
            ),
        )
    };
    assert_eq!(verify(&grow("0")), "proved");
    // Bytes past the old size are uninitialized after growth.
    assert_eq!(verify(&grow("5")), "counterexample");
    let zeroed = target(
        "",
        "start:\n %p = call ptr @__rust_alloc_zeroed(i64 8, i64 8)\n %l = load i64, ptr %p, align 8\n %v = icmp eq i64 %l, 0\n ret i1 %v",
    );
    assert_eq!(verify(&zeroed), "proved");
}
#[test]
fn stored_pointers_keep_provenance_until_overwritten() {
    let slot = |overwrite: &str| {
        target(
            "i8 %x",
            &format!(
                "start:\n %v = alloca i8, align 1\n store i8 %x, ptr %v, align 1\n %s = alloca ptr, align 8\n store ptr %v, ptr %s, align 8\n{overwrite}\n %p = load ptr, ptr %s, align 8\n %l = load i8, ptr %p, align 1\n %r = icmp eq i8 %l, %x\n ret i1 %r"
            ),
        )
    };
    assert_eq!(verify(&slot("")), "proved");
    assert_eq!(verify(&slot(" store i64 5, ptr %s, align 8")), "unknown");
}
#[test]
fn byte_copies_fills_and_comparisons() {
    let copy = target(
        "i32 %x",
        "start:\n %a = alloca i32, align 4\n %b = alloca i32, align 4\n store i32 %x, ptr %a, align 4\n call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b, ptr align 4 %a, i64 4, i1 false)\n %l = load i32, ptr %b, align 4\n %v = icmp eq i32 %l, %x\n ret i1 %v",
    );
    assert_eq!(verify(&copy), "proved");
    let overlap = target(
        "",
        "start:\n %a = alloca [8 x i8], align 1\n call void @llvm.memset.p0.i64(ptr %a, i8 0, i64 8, i1 false)\n %b = getelementptr inbounds i8, ptr %a, i64 2\n call void @llvm.memcpy.p0.p0.i64(ptr %b, ptr %a, i64 4, i1 false)\n ret i1 true",
    );
    assert_eq!(verify(&overlap), "counterexample");
    let moved = overlap.replace("llvm.memcpy", "llvm.memmove");
    assert_eq!(verify(&moved), "proved");
    let compare = |second: &str, expect: &str| {
        target(
            "i16 %x",
            &format!(
                "start:\n %a = alloca i16, align 2\n %b = alloca i16, align 2\n store i16 %x, ptr %a, align 2\n store i16 {second}, ptr %b, align 2\n %c = call i32 @bcmp(ptr %a, ptr %b, i64 2)\n %v = icmp {expect} i32 %c, 0\n ret i1 %v"
            ),
        )
    };
    assert_eq!(verify(&compare("%x", "eq")), "proved");
    assert_eq!(verify(&compare("%x", "ne")), "counterexample");
    let order = target(
        "",
        "start:\n %a = alloca i8, align 1\n %b = alloca i8, align 1\n store i8 1, ptr %a, align 1\n store i8 2, ptr %b, align 1\n %c = call i32 @memcmp(ptr %a, ptr %b, i64 1)\n %v = icmp slt i32 %c, 0\n ret i1 %v",
    );
    assert_eq!(verify(&order), "proved");
}
#[test]
fn symbolic_copy_length_is_bounded_by_the_objects() {
    let copy = target(
        "i64 %n, i8 %x",
        "start:\n %small = icmp ule i64 %n, 4\n br i1 %small, label %go, label %done\ngo:\n %a = alloca [4 x i8], align 1\n %b = alloca [4 x i8], align 1\n call void @llvm.memset.p0.i64(ptr %a, i8 %x, i64 4, i1 false)\n call void @llvm.memset.p0.i64(ptr %b, i8 0, i64 4, i1 false)\n call void @llvm.memcpy.p0.p0.i64(ptr %b, ptr %a, i64 %n, i1 false)\n %l = load i8, ptr %b, align 1\n %some = icmp ne i64 %n, 0\n %want = select i1 %some, i8 %x, i8 0\n %v = icmp eq i8 %l, %want\n ret i1 %v\ndone:\n ret i1 true",
    );
    assert_eq!(verify(&copy), "proved");
    let too_long = copy.replace("icmp ule i64 %n, 4", "icmp ule i64 %n, 5");
    assert_eq!(verify(&too_long), "counterexample");
}
#[test]
fn dangling_constants_are_zero_size_objects() {
    let compare = target(
        "",
        "start:\n %v = icmp ne ptr inttoptr (i64 1 to ptr), null\n ret i1 %v",
    );
    assert_eq!(verify(&compare), "proved");
    let read = target(
        "",
        "start:\n %l = load i8, ptr inttoptr (i64 1 to ptr), align 1\n %v = icmp eq i8 %l, 0\n ret i1 %v",
    );
    assert_eq!(verify(&read), "counterexample");
}
#[test]
fn aggregates_insert_extract_and_select() {
    let pair = target(
        "i8 %x, i1 %c",
        "start:\n %a = insertvalue { i8, i8 } poison, i8 %x, 0\n %b = insertvalue { i8, i8 } %a, i8 7, 1\n %z = insertvalue { i8, i8 } zeroinitializer, i8 7, 0\n %s = select i1 %c, { i8, i8 } %b, { i8, i8 } %z\n %f = extractvalue { i8, i8 } %s, 1\n %g = extractvalue { i8, i8 } %s, 0\n %e = icmp ule i8 %f, 7\n ret i1 %e",
    );
    assert_eq!(verify(&pair), "proved");
    // Reading the never-initialized field of a poison aggregate is observed.
    let poison = target(
        "i8 %x",
        "start:\n %a = insertvalue { i8, i8 } poison, i8 %x, 0\n %f = extractvalue { i8, i8 } %a, 1\n %e = icmp eq i8 %f, %f\n ret i1 %e",
    );
    assert_eq!(verify(&poison), "counterexample");
}
#[test]
fn allocator_symbols_parse_from_v0_mangling() {
    use crate::symbols::v0_segments;
    assert_eq!(
        v0_segments("_RNvCs1Y7DaGC1cwg_7___rustc12___rust_alloc"),
        Some(vec!["__rustc", "__rust_alloc"])
    );
    assert_eq!(
        v0_segments("_RNvNtCs1234_4core6option13unwrap_failed"),
        Some(vec!["core", "option", "unwrap_failed"])
    );
    assert_eq!(v0_segments("_RNvMs_NtCs1_4core3fmt"), None);
}
#[test]
fn poison_pointers_fail_only_when_used() {
    let guarded = |guard: &str| {
        target(
            "i1 %c",
            &format!(
                "start:\n %v = alloca i8, align 1\n store i8 1, ptr %v, align 1\n %p = select i1 %c, ptr %v, ptr poison\n br i1 {guard}, label %use, label %skip\nuse:\n %l = load i8, ptr %p, align 1\n %r = icmp eq i8 %l, 1\n ret i1 %r\nskip:\n ret i1 true"
            ),
        )
    };
    assert_eq!(verify(&guarded("%c")), "proved");
    assert_ne!(verify(&guarded("true")), "proved");
}
#[test]
fn typed_gep_scales_indices_and_rejects_wrapping() {
    let access = |bound: &str, index: &str| {
        target(
            "i64 %i, i32 %x",
            &format!(
                "start:\n %ok = icmp ult i64 %i, {bound}\n br i1 %ok, label %go, label %done\ngo:\n %a = alloca [16 x i8], align 4\n %p = getelementptr inbounds i32, ptr %a, i64 {index}\n store i32 %x, ptr %p, align 4\n %l = load i32, ptr %p, align 4\n %v = icmp eq i32 %l, %x\n ret i1 %v\ndone:\n ret i1 true"
            ),
        )
    };
    assert_eq!(verify(&access("4", "%i")), "proved");
    assert_eq!(verify(&access("5", "%i")), "counterexample");
    let array = target(
        "i64 %i",
        "start:\n %ok = icmp ult i64 %i, 3\n br i1 %ok, label %go, label %done\ngo:\n %a = alloca [6 x i8], align 2\n %p = getelementptr inbounds [3 x i16], ptr %a, i64 0, i64 %i\n store i16 7, ptr %p, align 2\n %l = load i16, ptr %p, align 2\n %v = icmp eq i16 %l, 7\n ret i1 %v\ndone:\n ret i1 true",
    );
    assert_eq!(verify(&array), "proved");
    // 2^62 * 4 wraps to 0: in bounds only by wrapping, so poison, not valid.
    let wrapped = target(
        "",
        "start:\n %a = alloca [16 x i8], align 4\n %p = getelementptr inbounds i32, ptr %a, i64 4611686018427387904\n store i32 1, ptr %p, align 4\n ret i1 true",
    );
    assert_eq!(verify(&wrapped), "counterexample");
}
#[test]
fn alignment_is_checked_on_the_address() {
    // core::slice::memchr on 16+ bytes: align_offset, then aligned words of
    // a byte string declared `align 1`. Was unknown ("allocation alignment
    // is insufficient"); the address is aligned on every placement.
    let s = "@s = private unnamed_addr constant [24 x i8] c\"aaaaaaaaaaaaaaaaaaaaaaaa\", align 1\n";
    let word = "%r = icmp eq i64 %w, 7016996765293437281\n ret i1 %r";
    let rounded = target(
        "",
        &format!(
            "start:\n %a = ptrtoint ptr @s to i64\n %u = add i64 %a, 7\n %b = and i64 %u, -8\n %o = sub i64 %b, %a\n %p = getelementptr inbounds i8, ptr @s, i64 %o\n %w = load i64, ptr %p, align 8\n {word}"
        ),
    );
    assert_eq!(verify(&format!("{s}{rounded}")), "proved");
    // Checked by the program itself before the read.
    let checked = target(
        "",
        &format!(
            "start:\n %a = ptrtoint ptr @s to i64\n %m = and i64 %a, 7\n %z = icmp eq i64 %m, 0\n br i1 %z, label %go, label %done\ngo:\n %w = load i64, ptr @s, align 8\n {word}\ndone:\n ret i1 true"
        ),
    );
    assert_eq!(verify(&format!("{s}{checked}")), "proved");
    // Unchecked, most placements make the read misaligned.
    let unchecked = target(
        "",
        &format!("start:\n %w = load i64, ptr @s, align 8\n {word}"),
    );
    let verdict = run(&format!("{s}{unchecked}")).expect("verdict");
    assert_eq!(verdict.status, "counterexample");
    // The witness is a placement: the model names the address it chose,
    // also when only the path (not the failing condition) depends on it.
    assert!(verdict.model.contains("placement: "), "{}", verdict.model);
    let branch = target(
        "",
        "start:\n %a = ptrtoint ptr @s to i64\n %m = and i64 %a, 7\n %z = icmp ne i64 %m, 0\n br i1 %z, label %bad, label %good\nbad:\n ret i1 false\ngood:\n ret i1 true",
    );
    let verdict = run(&format!("{s}{branch}")).expect("verdict");
    assert_eq!(verdict.status, "counterexample");
    assert!(verdict.model.contains("placement: "), "{}", verdict.model);
    // A definition without `align` keeps its type's ABI alignment.
    let implicit = "@g = internal global i64 7\n";
    let read = target(
        "",
        "start:\n %w = load i64, ptr @g, align 8\n %r = icmp eq i64 %w, 7\n ret i1 %r",
    );
    assert_eq!(verify(&format!("{implicit}{read}")), "proved");
}
#[test]
fn review_regressions_heap() {
    // Growing a zeroed allocation leaves the new bytes uninitialized.
    let grown = target(
        "",
        "start:\n %p = call ptr @__rust_alloc_zeroed(i64 1, i64 1)\n %q = call ptr @__rust_realloc(ptr %p, i64 1, i64 1, i64 2)\n %r = getelementptr inbounds i8, ptr %q, i64 1\n %x = load i8, ptr %r, align 1\n %ok = icmp eq i8 %x, 0\n ret i1 %ok",
    );
    assert_eq!(verify(&grown), "counterexample");
    // Shrinking then growing does not resurrect the discarded bytes.
    let resurrect = target(
        "",
        "start:\n %p = call ptr @__rust_alloc_zeroed(i64 4, i64 1)\n %q = call ptr @__rust_realloc(ptr %p, i64 4, i64 1, i64 1)\n %s = call ptr @__rust_realloc(ptr %q, i64 1, i64 1, i64 4)\n %r = getelementptr inbounds i8, ptr %s, i64 3\n %x = load i8, ptr %r, align 1\n %ok = icmp eq i8 %x, 0\n ret i1 %ok",
    );
    assert_eq!(verify(&resurrect), "counterexample");
    // `nusw` alone carries address obligations that are not modeled.
    let nusw = target(
        "",
        "start:\n %a = alloca i8, align 1\n %q = getelementptr nusw i8, ptr %a, i64 1\n %ok = icmp ne ptr %q, null\n ret i1 %ok",
    );
    assert_eq!(verify(&nusw), "unknown");
    // A call-site `align` promise on a byte operation must hold.
    let misaligned = target(
        "",
        "start:\n %p = alloca [4 x i8], align 4\n %q = getelementptr inbounds i8, ptr %p, i64 1\n call void @llvm.memset.p0.i64(ptr align 4 %q, i8 0, i64 1, i1 false)\n ret i1 true",
    );
    assert_eq!(verify(&misaligned), "counterexample");
    // memcpy with identical source and destination is allowed.
    let same = target(
        "",
        "start:\n %p = alloca i8, align 1\n store i8 7, ptr %p, align 1\n call void @llvm.memcpy.p0.p0.i64(ptr %p, ptr %p, i64 1, i1 false)\n %l = load i8, ptr %p, align 1\n %ok = icmp eq i8 %l, 7\n ret i1 %ok",
    );
    assert_eq!(verify(&same), "proved");
    // A poison alignment argument is undefined behavior.
    let poison_align = target(
        "",
        "start:\n %a = lshr exact i64 17, 1\n %p = call ptr @__rust_alloc(i64 1, i64 %a)\n ret i1 true",
    );
    assert_eq!(verify(&poison_align), "counterexample");
}
/// Verifies with a second, library-like module `lib` also callable.
fn verify_two(user: &str, library: &str) -> String {
    let user = Module::new("user", user.to_owned(), false).expect("user parses");
    let library = Module::new("lib", library.to_owned(), false).expect("lib parses");
    let entry = user.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(user));
    engine.modules.push(Rc::new(library));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
#[test]
fn review_regressions_calls() {
    // Private helpers of two modules must not share a body.
    let user = "define internal i1 @helper() {\nentry:\n ret i1 true\n}\ndefine i1 @phage_target() {\nstart:\n %a = call i1 @helper()\n %b = call i1 @wrap()\n %v = and i1 %a, %b\n ret i1 %v\n}\ndeclare i1 @wrap()\n";
    let library = "define internal i1 @helper() {\nentry:\n ret i1 false\n}\ndefine i1 @wrap() {\nentry:\n %r = call i1 @helper()\n ret i1 %r\n}\n";
    assert_eq!(verify_two(user, library), "counterexample");
    // A callee's stack object dies at its return.
    let escape = "define ptr @leak() {\nentry:\n %p = alloca i8, align 1\n store i8 7, ptr %p, align 1\n ret ptr %p\n}\n";
    let entry = target(
        "",
        "start:\n %p = call ptr @leak()\n %l = load i8, ptr %p, align 1\n %v = icmp eq i8 %l, 7\n ret i1 %v",
    );
    assert_eq!(verify(&format!("{escape}{entry}")), "counterexample");
    // Poison passed to a noundef parameter is undefined behavior.
    let sink = "define void @sink(i8 noundef %x) {\nentry:\n ret void\n}\n";
    let call = target(
        "",
        "start:\n call void @sink(i8 noundef poison)\n ret i1 true",
    );
    assert_eq!(verify(&format!("{sink}{call}")), "counterexample");
    // The definition's own `noundef` binds even without the call-site attribute.
    let plain = call.replace("i8 noundef poison", "i8 poison");
    assert_eq!(verify(&format!("{sink}{plain}")), "counterexample");
    let open = "define void @open(i8 %x) {\nentry:\n ret void\n}\n";
    let loose = target("", "start:\n call void @open(i8 poison)\n ret i1 true");
    assert_eq!(verify(&format!("{open}{loose}")), "proved");
    // Returning poison from a noundef function is undefined behavior.
    let give = "define noundef i8 @give() {\nentry:\n ret i8 poison\n}\n";
    let use_give = target("", "start:\n %x = call i8 @give()\n ret i1 true");
    assert_eq!(verify(&format!("{give}{use_give}")), "counterexample");
    // dereferenceable(N) requires N accessible bytes.
    let touch = "define void @touch(ptr dereferenceable(8) %p) {\nentry:\n ret void\n}\n";
    let short = target(
        "",
        "start:\n %p = alloca [4 x i8], align 1\n call void @touch(ptr dereferenceable(8) %p)\n ret i1 true",
    );
    assert_eq!(verify(&format!("{touch}{short}")), "counterexample");
    // Signed division of a poison dividend is poison, not undefined behavior.
    let division = target("", "start:\n %q = sdiv i8 poison, 1\n ret i1 true");
    assert_eq!(verify(&division), "proved");
}
#[test]
fn diverging_calls_mean_the_property_cannot_return() {
    let spin = "define void @spin() #1 {\nentry:\n br label %loop\nloop:\n br label %loop\n}\n";
    let wander = "define void @wander() {\nentry:\n br label %loop\nloop:\n br label %loop\n}\n";
    let entry = |callee: &str| {
        format!(
            "{}\nattributes #1 = {{ noreturn }}\n",
            target(
                "i8 %x",
                &format!(
                    "start:\n %c = icmp eq i8 %x, 3\n br i1 %c, label %go, label %ok\ngo:\n call void @{callee}()\n ret i1 true\nok:\n ret i1 true"
                ),
            )
        )
    };
    // `noreturn`: this path can never return true, reached at x = 3.
    assert_eq!(
        verify(&format!("{spin}{}", entry("spin"))),
        "counterexample"
    );
    // Without the attribute the body is executed and the loop hits the bound.
    assert_eq!(verify(&format!("{wander}{}", entry("wander"))), "unknown");
}
#[test]
fn vtables_dispatch_indirect_calls() {
    let methods = "define internal noundef range(i8 0, 10) i8 @one(ptr %self) {\nentry:\n ret i8 1\n}\ndefine internal i8 @two(ptr %self) {\nentry:\n ret i8 2\n}\n";
    let tables = "@vt1 = private unnamed_addr constant <{ [16 x i8], ptr }> <{ [16 x i8] c\"\\00\\00\\00\\00\\00\\00\\00\\00\\01\\00\\00\\00\\00\\00\\00\\00\", ptr @one }>, align 8\n@vt2 = private unnamed_addr constant <{ [16 x i8], ptr }> <{ [16 x i8] c\"\\00\\00\\00\\00\\00\\00\\00\\00\\01\\00\\00\\00\\00\\00\\00\\00\", ptr @two }>, align 8\n";
    let dispatch = |check: &str| {
        target(
            "i1 %c",
            &format!(
                "start:\n %t = select i1 %c, ptr @vt1, ptr @vt2\n %s = getelementptr inbounds i8, ptr %t, i64 16\n %f = load ptr, ptr %s, align 8\n %d = load ptr, ptr %t, align 8\n %none = icmp eq ptr %d, null\n %r = call i8 %f(ptr null)\n{check}"
            ),
        )
    };
    let exact = dispatch(
        " %w = select i1 %c, i8 1, i8 2\n %e = icmp eq i8 %r, %w\n %v = and i1 %e, %none\n ret i1 %v",
    );
    assert_eq!(verify(&format!("{tables}{methods}{exact}")), "proved");
    let wrong = dispatch(" %v = icmp eq i8 %r, 1\n ret i1 %v");
    assert_eq!(
        verify(&format!("{tables}{methods}{wrong}")),
        "counterexample"
    );
    // A displaced function pointer or a data pointer is not callable.
    let displaced = target(
        "",
        "start:\n %f = getelementptr i8, ptr @one, i64 1\n %r = call i8 %f(ptr null)\n ret i1 true",
    );
    assert_ne!(verify(&format!("{methods}{displaced}")), "proved");
    let data = target(
        "",
        "start:\n %p = alloca i8, align 1\n %r = call i8 %p(ptr null)\n ret i1 true",
    );
    assert_eq!(verify(&data), "unknown");
}
#[test]
fn selects_of_undef_and_same_object_pointers() {
    // A selected-but-unobserved undef is harmless; observing it fails.
    let unused = target(
        "i1 %c, i64 %x",
        "start:\n %s = select i1 %c, i64 undef, i64 %x\n ret i1 true",
    );
    assert_eq!(verify(&unused), "proved");
    let observed = target(
        "i1 %c, i64 %x",
        "start:\n %s = select i1 %c, i64 undef, i64 %x\n %v = icmp eq i64 %s, %s\n ret i1 %v",
    );
    assert_eq!(verify(&observed), "counterexample");
    // Two pointers into one object need no path split.
    let same = target(
        "i1 %c",
        "start:\n %a = alloca [2 x i8], align 1\n store i8 5, ptr %a, align 1\n %b = getelementptr inbounds i8, ptr %a, i64 1\n store i8 6, ptr %b, align 1\n %p = select i1 %c, ptr %a, ptr %b\n %x = load i8, ptr %p, align 1\n %v = icmp uge i8 %x, 5\n ret i1 %v",
    );
    assert_eq!(verify(&same), "proved");
}
#[test]
fn review_recheck_regressions() {
    // noreturn is read in the callee's scope: another module's private
    // noreturn function of the same name does not make this call diverge.
    let user = "define internal i1 @helper() {\nentry:\n ret i1 true\n}\ndefine i1 @phage_target() {\nstart:\n %r = call i1 @helper()\n ret i1 %r\n}\n";
    let library = "define internal void @helper() #1 {\nentry:\n unreachable\n}\nattributes #1 = { noreturn }\n";
    assert_eq!(verify_two(user, library), "proved");
    // Contracts bind modeled calls too, even with a zero length.
    let memset = target(
        "",
        "start:\n %p = alloca i8, align 1\n call void @llvm.memset.p0.i64(ptr dereferenceable(8) %p, i8 0, i64 0, i1 false)\n ret i1 true",
    );
    assert_eq!(verify(&memset), "counterexample");
    // A call-site noundef result binds a callee that does not promise it.
    let give = "define i8 @give() {\nentry:\n ret i8 poison\n}\n";
    let strict = target("", "start:\n %x = call noundef i8 @give()\n ret i1 true");
    assert_eq!(verify(&format!("{give}{strict}")), "counterexample");
    let relaxed = target("", "start:\n %x = call i8 @give()\n ret i1 true");
    assert_eq!(verify(&format!("{give}{relaxed}")), "proved");
    // A poison field makes a whole noundef aggregate undefined.
    let pair = "define void @take({ i8, i8 } noundef %p) {\nentry:\n ret void\n}\n";
    let partial = target(
        "i8 %x",
        "start:\n %a = insertvalue { i8, i8 } poison, i8 %x, 0\n call void @take({ i8, i8 } %a)\n ret i1 true",
    );
    assert_eq!(verify(&format!("{pair}{partial}")), "counterexample");
}
#[test]
fn final_recheck_regressions() {
    // An undef argument violates noundef.
    let sink = "define void @sink(i8 %x) {\nentry:\n ret void\n}\n";
    let undef_arg = target(
        "",
        "start:\n call void @sink(i8 noundef undef)\n ret i1 true",
    );
    assert_eq!(verify(&format!("{sink}{undef_arg}")), "counterexample");
    // Contracts bind lifetime markers too.
    let lifetime = target(
        "",
        "start:\n %a = alloca i8, align 1\n %p = select i1 poison, ptr %a, ptr %a\n call void @llvm.lifetime.end.p0(i64 1, ptr noundef %p)\n ret i1 true",
    );
    assert_eq!(verify(&lifetime), "counterexample");
    let new_lifetime = lifetime.replace("i64 1, ptr noundef %p", "ptr noundef %p");
    assert_eq!(verify(&new_lifetime), "counterexample");
    // An inbounds constant GEP past its target is a poison pointer.
    let relocated = |addend: &str| {
        format!(
            "@s = private constant [1 x i8] c\"A\", align 1\n@p = private constant <{{ ptr }}> <{{ ptr getelementptr inbounds (i8, ptr @s, i64 {addend}) }}>, align 8\n{}",
            target(
                "",
                "start:\n %q = load ptr, ptr @p, align 8\n %v = icmp ne ptr %q, null\n ret i1 %v"
            )
        )
    };
    assert_eq!(verify(&relocated("1")), "proved");
    assert_eq!(verify(&relocated("2")), "counterexample");
    // A global whose pointer cannot be resolved is unavailable, not null.
    let unresolved = format!(
        "@missing = external global i8\n@p = private constant <{{ ptr }}> <{{ ptr @missing }}>, align 8\n{}",
        target(
            "",
            "start:\n %q = load ptr, ptr @p, align 8\n %v = icmp ne ptr %q, null\n ret i1 %v"
        )
    );
    assert_eq!(verify(&unresolved), "unknown");
    // Struct padding in a constant is not a defined zero.
    let padded = |offset: &str| {
        format!(
            "@g = private constant {{ i8, i32 }} {{ i8 1, i32 2 }}, align 4\n{}",
            target(
                "",
                &format!(
                    "start:\n %p = getelementptr inbounds i8, ptr @g, i64 {offset}\n %x = load i8, ptr %p, align 1\n %v = icmp eq i8 %x, %x\n ret i1 %v"
                ),
            )
        )
    };
    assert_eq!(verify(&padded("0")), "proved");
    assert_eq!(verify(&padded("1")), "counterexample");
}
