//! End-to-end controls for the runtime semantics std code relies on:
//! mutable and thread-local statics, atomics, getrandom, integer vectors and
//! pointer provenance through integers. Each positive control has a failing
//! or unknown twin. Real Z3 is required.

use crate::{engine::Engine, ir::Module};
use std::rc::Rc;

/// Verifies `@phage_target` in `user`, with `library` also loaded.
fn verify_with(user: &str, library: Option<&str>) -> String {
    let user = Module::new("user", user.to_owned(), false).expect("user parses");
    let entry = user.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(user));
    if let Some(library) = library {
        let library = Module::new("lib", library.to_owned(), false).expect("lib parses");
        engine.modules.push(Rc::new(library));
    }
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
fn verify(user: &str) -> String {
    verify_with(user, None)
}
fn target(parameters: &str, blocks: &str) -> String {
    format!("define i1 @phage_target({parameters}) {{\n{blocks}\n}}\n")
}

#[test]
fn pointers_without_provenance_compare_by_address() {
    let round_trip = "start:\n %p = alloca i8, align 1\n %a = ptrtoint ptr %p to i64\n %q = inttoptr i64 %a to ptr\n";
    // Not null-folded: the round trip is the same address.
    assert_eq!(
        verify(&target(
            "",
            &format!("{round_trip} %v = icmp ne ptr %p, %q\n ret i1 %v")
        )),
        "counterexample"
    );
    assert_eq!(
        verify(&target(
            "",
            &format!("{round_trip} %v = icmp eq ptr %p, %q\n ret i1 %v")
        )),
        "proved"
    );
    // ptrtoint exposed the allocation, so the round trip reaches it
    // (exposed provenance) and the store is read back...
    assert_eq!(
        verify(&target(
            "",
            &format!(
                "{round_trip} store i8 1, ptr %q, align 1\n %l = load i8, ptr %p, align 1\n %v = icmp eq i8 %l, 1\n ret i1 %v"
            )
        )),
        "proved"
    );
    // ...with bounds still checked past the end...
    assert_eq!(
        verify(&target(
            "",
            "start:\n %p = alloca i8, align 1\n %a = ptrtoint ptr %p to i64\n %b = add i64 %a, 8\n %q = inttoptr i64 %b to ptr\n store i8 1, ptr %q, align 1\n ret i1 true"
        )),
        "counterexample"
    );
    // ...while an unrelated integer reaches no object.
    assert_eq!(
        verify(&target(
            "i64 %x",
            "start:\n %q = inttoptr i64 %x to ptr\n store i8 1, ptr %q, align 1\n ret i1 true"
        )),
        "unknown"
    );
    // Comparing with null compares the address itself.
    assert_eq!(
        verify(&target(
            "i64 %x",
            "start:\n %q = inttoptr i64 %x to ptr\n %n = icmp eq ptr %q, null\n %z = icmp eq i64 %x, 0\n %v = icmp eq i1 %n, %z\n ret i1 %v"
        )),
        "proved"
    );
}

#[test]
fn integer_copies_move_provenance_only_when_exact() {
    let setup = "start:\n %x = alloca i8, align 1\n store i8 7, ptr %x, align 1\n %a = alloca ptr, align 8\n %b = alloca ptr, align 8\n store ptr %x, ptr %a, align 8\n %i = load i64, ptr %a, align 8\n";
    let tail = " %p = load ptr, ptr %b, align 8\n %v = load i8, ptr %p, align 1\n %r = icmp eq i8 %v, 7\n ret i1 %r";
    assert_eq!(
        verify(&target(
            "",
            &format!("{setup} store i64 %i, ptr %b, align 8\n{tail}")
        )),
        "proved"
    );
    // Arithmetic produces a new integer: no provenance is manufactured.
    assert_eq!(
        verify(&target(
            "",
            &format!("{setup} %j = add i64 %i, 0\n store i64 %j, ptr %b, align 8\n{tail}")
        )),
        "unknown"
    );
}

#[test]
fn atomics_are_sequential_read_modify_writes() {
    let slot = "start:\n %p = alloca i32, align 4\n store i32 5, ptr %p, align 4\n";
    let strong = format!(
        "{slot} %r = cmpxchg ptr %p, i32 5, i32 7 seq_cst seq_cst, align 4\n %ok = extractvalue {{ i32, i1 }} %r, 1\n %n = load atomic i32, ptr %p acquire, align 4\n %seven = icmp eq i32 %n, 7\n %v = and i1 %ok, %seven\n ret i1 %v"
    );
    assert_eq!(verify(&target("", &strong)), "proved");
    // A weak exchange may fail spuriously.
    let weak = format!(
        "{slot} %r = cmpxchg weak ptr %p, i32 5, i32 7 seq_cst seq_cst, align 4\n %ok = extractvalue {{ i32, i1 }} %r, 1\n ret i1 %ok"
    );
    assert_eq!(verify(&target("", &weak)), "counterexample");
    let fetch = format!(
        "{slot} %o = atomicrmw umax ptr %p, i32 %y monotonic, align 4\n %n = load i32, ptr %p, align 4\n %old = icmp eq i32 %o, 5\n %ge = icmp uge i32 %n, %y\n %v = and i1 %old, %ge\n ret i1 %v"
    );
    assert_eq!(verify(&target("i32 %y", &fetch)), "proved");
    // An omitted alignment is the value's size, so this access is misaligned.
    assert_eq!(
        verify(&target(
            "",
            "start:\n %buf = alloca [16 x i8], align 8\n %p = getelementptr i8, ptr %buf, i64 1\n store i64 0, ptr %p, align 1\n %o = atomicrmw add ptr %p, i64 1 monotonic\n ret i1 true"
        )),
        "counterexample"
    );
    // Read-modify-write of uninitialized memory is refused.
    assert_eq!(
        verify(&target(
            "",
            "start:\n %p = alloca i32, align 4\n %o = atomicrmw add ptr %p, i32 1 seq_cst, align 4\n ret i1 true"
        )),
        "counterexample"
    );
}

#[test]
fn mutable_and_thread_local_statics_start_at_their_initializers() {
    let global = "@g = internal global i32 5, align 4\n";
    let read = "start:\n %n = load i32, ptr @g, align 4\n";
    assert_eq!(
        verify(&format!(
            "{global}{}",
            target("", &format!("{read} %v = icmp eq i32 %n, 5\n ret i1 %v"))
        )),
        "proved"
    );
    // Writable: a store changes what later loads see.
    assert_eq!(
        verify(&format!(
            "{global}{}",
            target(
                "i32 %x",
                "start:\n store i32 %x, ptr @g, align 4\n %n = load i32, ptr @g, align 4\n %v = icmp eq i32 %n, 5\n ret i1 %v"
            )
        )),
        "counterexample"
    );
    // `undef` parts of an initializer are uninitialized, not zero.
    let lazy = "@h = internal global <{ [4 x i8], [4 x i8] }> <{ [4 x i8] undef, [4 x i8] zeroinitializer }>, align 4\n";
    assert_eq!(
        verify(&format!(
            "{lazy}{}",
            target(
                "",
                "start:\n %q = getelementptr inbounds i8, ptr @h, i64 4\n %n = load i32, ptr %q, align 4\n %v = icmp eq i32 %n, 0\n ret i1 %v"
            )
        )),
        "proved"
    );
    assert_eq!(
        verify(&format!(
            "{lazy}{}",
            target(
                "",
                "start:\n %n = load i32, ptr @h, align 4, !noundef !0\n ret i1 true"
            )
        )),
        "counterexample"
    );
    // A definition the final link may replace is not trusted.
    assert_eq!(
        verify(&format!(
            "@w = weak global i32 1, align 4\n{}",
            target(
                "",
                "start:\n %n = load i32, ptr @w, align 4\n %v = icmp eq i32 %n, 1\n ret i1 %v"
            )
        )),
        "unknown"
    );
    let tls = "@t = internal thread_local global i8 3, align 1\ndeclare ptr @llvm.threadlocal.address.p0(ptr)\n";
    assert_eq!(
        verify(&format!(
            "{tls}{}",
            target(
                "",
                "start:\n %p = call ptr @llvm.threadlocal.address.p0(ptr @t)\n %n = load i8, ptr %p, align 1\n %v = icmp eq i8 %n, 3\n ret i1 %v"
            )
        )),
        "proved"
    );
}

#[test]
fn declared_globals_resolve_to_the_exported_definition() {
    let user = format!(
        "@G = external global i32\n{}",
        target(
            "",
            "start:\n %n = load i32, ptr @G, align 4\n %v = icmp eq i32 %n, 9\n ret i1 %v"
        )
    );
    assert_eq!(
        verify_with(&user, Some("@G = global i32 9, align 4\n")),
        "proved"
    );
    assert_eq!(
        verify_with(&user, Some("@G = global i32 8, align 4\n")),
        "counterexample"
    );
    // A private definition elsewhere is not the declared symbol.
    assert_eq!(
        verify_with(&user, Some("@G = internal global i32 9, align 4\n")),
        "unknown"
    );
    // A definition here that the parser rejects (i1 has no byte layout)
    // never falls back to another module's global of the same name.
    let rejected = format!(
        "@G = private global i1 true\n{}",
        target(
            "",
            "start:\n %n = load i32, ptr @G, align 4\n %v = icmp eq i32 %n, 9\n ret i1 %v"
        )
    );
    assert_eq!(
        verify_with(&rejected, Some("@G = global i32 9, align 4\n")),
        "unknown"
    );
    // Mutable ODR copies are one object after linking: refused.
    let odr = "@c = linkonce_odr global i8 0, align 1\n";
    assert_eq!(
        verify_with(
            &format!(
                "{odr}declare i8 @read()\n{}",
                target(
                    "",
                    "start:\n store i8 1, ptr @c, align 1\n %n = call i8 @read()\n %v = icmp eq i8 %n, 0\n ret i1 %v"
                )
            ),
            Some(&format!(
                "{odr}define i8 @read() {{\nstart:\n %n = load i8, ptr @c, align 1\n ret i8 %n\n}}\n"
            ))
        ),
        "unknown"
    );
}

#[test]
fn getrandom_fills_the_buffer_with_arbitrary_bytes() {
    let declare = "declare i64 @getrandom(ptr, i64, i32)\n";
    let call =
        "start:\n %b = alloca i64, align 8\n %n = call i64 @getrandom(ptr %b, i64 8, i32 1)\n";
    assert_eq!(
        verify(&format!(
            "{declare}{}",
            target(
                "",
                &format!(
                    "{call} %k = load i64, ptr %b, align 8\n %v = icmp eq i64 %n, 8\n ret i1 %v"
                )
            )
        )),
        "proved"
    );
    assert_eq!(
        verify(&format!(
            "{declare}{}",
            target(
                "",
                &format!(
                    "{call} %k = load i64, ptr %b, align 8\n %v = icmp ne i64 %k, 0\n ret i1 %v"
                )
            )
        )),
        "counterexample"
    );
    assert_eq!(
        verify(&format!(
            "{declare}{}",
            target(
                "",
                "start:\n %b = alloca i64, align 8\n %n = call i64 @getrandom(ptr %b, i64 16, i32 1)\n ret i1 true"
            )
        )),
        "counterexample"
    );
}

/// Opaque mixing abstracts the value only: a random low byte under an
/// undefined high byte survives `or 0` and truncation (exact confirmation
/// refutes the abstract candidate). Twin: the undefined high byte is poison.
#[test]
fn opaque_mixing_keeps_per_byte_definedness() {
    let declare = "declare i64 @getrandom(ptr, i64, i32)\n";
    let body = |tail: &str| {
        format!(
            "{declare}{}",
            target(
                "",
                &format!(
                    "start:\n %b = alloca i16, align 2\n %n = call i64 @getrandom(ptr %b, i64 1, i32 1)\n %w = load i16, ptr %b, align 2\n %m = or i16 %w, 0\n %low = load i8, ptr %b, align 2\n{tail}"
                )
            )
        )
    };
    assert_eq!(
        verify(&body(
            " %t = trunc i16 %m to i8\n %v = icmp eq i8 %t, %low\n ret i1 %v"
        )),
        "proved"
    );
    assert_eq!(
        verify(&body(
            " %h = lshr i16 %m, 8\n %t = trunc i16 %h to i8\n %v = icmp eq i8 %t, %t\n ret i1 %v"
        )),
        "counterexample"
    );
}

#[test]
fn vectors_keep_lane_order_and_lane_poison() {
    // Broadcast, compare and movemask: all four sign bits equal x's sign.
    assert_eq!(
        verify(&target(
            "i8 %x",
            "start:\n %v = insertelement <4 x i8> poison, i8 %x, i64 0\n %s = shufflevector <4 x i8> %v, <4 x i8> poison, <4 x i32> zeroinitializer\n %c = icmp slt <4 x i8> %s, zeroinitializer\n %m = bitcast <4 x i1> %c to i4\n %all = icmp eq i4 %m, -1\n %neg = icmp slt i8 %x, 0\n %r = icmp eq i1 %all, %neg\n ret i1 %r"
        )),
        "proved"
    );
    // Lanes 1..3 stay poison without the broadcast.
    assert_eq!(
        verify(&target(
            "i8 %x",
            "start:\n %v = insertelement <4 x i8> poison, i8 %x, i64 0\n %b = bitcast <4 x i8> %v to i32\n %r = icmp eq i32 %b, %b\n ret i1 %r"
        )),
        "counterexample"
    );
    let lane = |index: u32| {
        target(
            "i32 %x",
            &format!(
                "start:\n %v = bitcast i32 %x to <4 x i8>\n %l = extractelement <4 x i8> %v, i64 {index}\n %t = trunc i32 %x to i8\n %r = icmp eq i8 %l, %t\n ret i1 %r"
            ),
        )
    };
    assert_eq!(verify(&lane(0)), "proved");
    assert_eq!(verify(&lane(3)), "counterexample");
    // Out of range is poison.
    assert_eq!(verify(&lane(4)), "counterexample");
    // Memory round trip in little-endian lane order.
    assert_eq!(
        verify(&target(
            "i64 %x",
            "start:\n %p = alloca [16 x i8], align 16\n %w = insertelement <2 x i64> splat (i64 0), i64 %x, i64 1\n store <2 x i64> %w, ptr %p, align 16\n %g = load <16 x i8>, ptr %p, align 16\n %b = extractelement <16 x i8> %g, i64 8\n %t = trunc i64 %x to i8\n %r = icmp eq i8 %b, %t\n ret i1 %r"
        )),
        "proved"
    );
}

#[test]
fn implicit_alignment_and_argument_attributes_are_checked() {
    // An omitted load/store alignment is the type's ABI alignment.
    assert_eq!(
        verify(&target(
            "",
            "start:\n %a = alloca [8 x i8], align 8\n %p = getelementptr i8, ptr %a, i64 1\n store i32 0, ptr %p\n ret i1 true"
        )),
        "counterexample"
    );
    let sink = "define void @sink(ptr noundef nonnull align 4 %p) {\nstart:\n ret void\n}\n";
    let call = |argument: &str| {
        format!(
            "{sink}{}",
            target(
                "",
                &format!(
                    "start:\n %a = alloca [8 x i8], align 8\n %q = getelementptr i8, ptr %a, i64 {argument}\n call void @sink(ptr %q)\n ret i1 true"
                )
            )
        )
    };
    assert_eq!(verify(&call("4")), "proved");
    // Misaligned for the callee's `align 4` with noundef: undefined behavior.
    assert_eq!(verify(&call("2")), "counterexample");
    assert_eq!(
        verify(&format!(
            "{sink}{}",
            target("", "start:\n call void @sink(ptr null)\n ret i1 true")
        )),
        "counterexample"
    );
    // A value outside a noundef range(...) is undefined behavior; the
    // range wraps when its bounds are reversed.
    let ranged = |bounds: &str, value: &str| {
        format!(
            "define void @digit(i8 noundef range(i8 {bounds}) %d) {{\nstart:\n ret void\n}}\n{}",
            target(
                "",
                &format!("start:\n call void @digit(i8 {value})\n ret i1 true")
            )
        )
    };
    assert_eq!(verify(&ranged("0, 10", "9")), "proved");
    assert_eq!(verify(&ranged("0, 10", "10")), "counterexample");
    assert_eq!(verify(&ranged("-2, 3", "-1")), "proved");
    assert_eq!(verify(&ranged("-2, 3", "5")), "counterexample");
    // Without noundef a violation only poisons the argument, which callee
    // models do not track: unknown rather than a proof.
    assert_eq!(
        verify(&format!(
            "define void @maybe(ptr nonnull %p) {{\nstart:\n ret void\n}}\n{}",
            target(
                "i1 %c",
                "start:\n %p = select i1 %c, ptr null, ptr null\n call void @maybe(ptr %p)\n ret i1 true"
            )
        )),
        "unknown"
    );
    // An argument that is already poison stays poison: nothing to check.
    assert_eq!(
        verify(&format!(
            "define void @maybe(ptr nonnull %p) {{\nstart:\n ret void\n}}\n{}",
            target(
                "",
                "start:\n call void @maybe(ptr nonnull align 1 poison)\n ret i1 true"
            )
        )),
        "proved"
    );
    // Indented top-level entities are refused rather than skipped.
    assert!(
        Module::new(
            "user",
            "  define i1 @f() {\nstart:\n ret i1 false\n}\n".into(),
            false
        )
        .is_err()
    );
}

#[test]
fn undefined_bytes_are_tracked_per_byte_and_poison_per_value() {
    let partial = "start:\n %s = alloca [8 x i8], align 8\n store i8 %x, ptr %s, align 8\n";
    let copied = "%w = load i64, ptr %s, align 8\n %d = alloca [8 x i8], align 8\n store i64 %w, ptr %d, align 8\n %y = load i8, ptr %d, align 8, !noundef !0\n %r = icmp eq i8 %y, %x\n ret i1 %r";
    // Copying a partly initialized struct keeps its defined byte defined.
    assert_eq!(
        verify(&target("i8 %x", &format!("{partial} {copied}"))),
        "proved"
    );
    // A poison byte poisons the whole wide load, hence the copy.
    let tainted = format!(
        "{partial} %p = add nuw i8 %x, 255\n %one = getelementptr inbounds i8, ptr %s, i64 1\n store i8 %p, ptr %one, align 1\n {copied}"
    );
    assert_eq!(verify(&target("i8 %x", &tainted)), "counterexample");
    // A byte packed into a pointer-typed slot survives ptrtoint and trunc.
    assert_eq!(
        verify(&target(
            "i8 %x",
            &format!(
                "{partial} %q = load ptr, ptr %s, align 8\n %i = ptrtoint ptr %q to i64\n %k = trunc i64 %i to i8\n %r = icmp eq i8 %k, %x\n ret i1 %r"
            )
        )),
        "proved"
    );
    // Constant shifts and masks extract exactly the defined bytes.
    let middle = "start:\n %s = alloca [8 x i8], align 8\n %two = getelementptr inbounds i8, ptr %s, i64 2\n store i16 %x, ptr %two, align 2\n %w = load i64, ptr %s, align 8\n";
    assert_eq!(
        verify(&target(
            "i16 %x",
            &format!(
                "{middle} %h = lshr i64 %w, 16\n %t = trunc i64 %h to i16\n %r = icmp eq i16 %t, %x\n ret i1 %r"
            )
        )),
        "proved"
    );
    assert_eq!(
        verify(&target(
            "i16 %x",
            &format!(
                "{middle} %h = lshr i64 %w, 8\n %t = trunc i64 %h to i16\n %r = icmp eq i16 %t, %t\n ret i1 %r"
            )
        )),
        "counterexample"
    );
    assert_eq!(
        verify(&target(
            "i8 %x",
            &format!(
                "{partial} %w = load i64, ptr %s, align 8\n %m = and i64 %w, 255\n %z = zext i8 %x to i64\n %r = icmp eq i64 %m, %z\n ret i1 %r"
            )
        )),
        "proved"
    );
}

#[test]
fn select_and_bitcast_keep_per_byte_definedness() {
    let low_byte = "start:\n %p = alloca i16, align 2\n store i8 7, ptr %p, align 1\n %x = load i16, ptr %p, align 2\n";
    assert_eq!(
        verify(&target(
            "i1 %c",
            &format!(
                "{low_byte} %s = select i1 %c, i16 %x, i16 7\n %lo = trunc i16 %s to i8\n %ok = icmp eq i8 %lo, 7\n ret i1 %ok"
            )
        )),
        "proved"
    );
    // The undefined high byte is still undefined after the select.
    assert_eq!(
        verify(&target(
            "i1 %c",
            &format!(
                "{low_byte} %s = select i1 %c, i16 %x, i16 7\n %hi = lshr i16 %s, 8\n %t = trunc i16 %hi to i8\n %ok = icmp eq i8 %t, %t\n ret i1 %ok"
            )
        )),
        "counterexample"
    );
    let lane = |index: u32| {
        target(
            "",
            &format!(
                "{low_byte} %v = bitcast i16 %x to <2 x i8>\n %l = extractelement <2 x i8> %v, i64 {index}\n %ok = icmp eq i8 %l, 7\n ret i1 %ok"
            ),
        )
    };
    assert_eq!(verify(&lane(0)), "proved");
    assert_eq!(verify(&lane(1)), "counterexample");
}

#[test]
fn oversized_literals_wrap_to_their_width() {
    // LLVM reads `i8 300` as 44: the comparison is false, never proved.
    assert_eq!(
        verify(&target("", "start:\n %r = icmp ne i8 300, 44\n ret i1 %r")),
        "counterexample"
    );
    assert_eq!(
        verify(&target("", "start:\n %r = icmp eq i8 300, 44\n ret i1 %r")),
        "proved"
    );
}

/// A surviving `llvm.is.constant` is lowered to false by code generation.
#[test]
fn is_constant_lowers_to_false() {
    let call = |tail: &str| {
        format!(
            "declare i1 @llvm.is.constant.i8(i8)\n{}",
            target(
                "i8 %x",
                &format!("start:\n %c = call i1 @llvm.is.constant.i8(i8 %x)\n{tail}")
            )
        )
    };
    assert_eq!(verify(&call(" ret i1 true")), "proved");
    assert_eq!(verify(&call(" ret i1 %c")), "counterexample");
    assert_eq!(verify(&call(" %n = xor i1 %c, true\n ret i1 %n")), "proved");
}

/// `invoke` (std is built with unwinding): the normal edge continues with
/// the result; a callee that cannot return normally is still a failure.
#[test]
fn invoke_continues_on_the_normal_edge() {
    let module = |callee_body: &str, check: &str| {
        format!(
            "define internal i8 @f(i8 %a) {{\nstart:\n{callee_body}\n}}\n{}",
            target(
                "i8 %x",
                &format!(
                    "start:\n %r = invoke i8 @f(i8 %x)\n          to label %ok unwind label %bad\nok:\n %p = phi i8 [ %r, %start ]\n{check}\nbad:\n %lp = landingpad {{ ptr, i32 }}\n          cleanup\n resume {{ ptr, i32 }} %lp"
                )
            )
        )
    };
    let identity = " ret i8 %a";
    assert_eq!(
        verify(&module(identity, " %v = icmp eq i8 %p, %x\n ret i1 %v")),
        "proved"
    );
    assert_eq!(
        verify(&module(identity, " %v = icmp ne i8 %p, 0\n ret i1 %v")),
        "counterexample"
    );
    assert_eq!(
        verify(&module(" unreachable", " ret i1 true")),
        "counterexample"
    );
}
