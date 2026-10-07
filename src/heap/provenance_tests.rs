//! Symbolic pointer-slot reads, whole integer copies, and controls that must
//! retain bounds, byte integrity, poison, lifetime and unsupported paths.

use crate::{engine::Engine, ir::Module};
use std::rc::Rc;

fn verify(body: &str, states: usize) -> String {
    let text = format!(
        "declare ptr @__rust_alloc(i64, i64)\n\
         declare ptr @__rust_realloc(ptr, i64, i64, i64)\n\
         declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)\n\
         declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)\n\
         declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)\n\
         declare void @llvm.lifetime.end.p0(i64, ptr)\n\
         define i1 @phage_target(i1 %pick, i1 %other) {{\n{body}\n}}"
    );
    let module = Module::new("user", text, false).expect("valid LLVM fixture");
    let entry = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, 16, states).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    match engine.verify(&entry) {
        Ok(v) => {
            eprintln!("{}: {}", v.status, v.detail);
            v.status
        }
        Err(e) => {
            eprintln!("error: {e}");
            "unknown".into()
        }
    }
}

fn fixture(change: &str, read: &str, expected: &str) -> String {
    format!(
        "start:
 %a = alloca i8, align 1
 %b = alloca i8, align 1
 store i8 7, ptr %a, align 1
 store i8 9, ptr %b, align 1
 %slots = alloca [24 x i8], align 8
 %second = getelementptr i8, ptr %slots, i64 8
 store ptr %a, ptr %slots, align 8
 store ptr %b, ptr %second, align 8
 {change}
 %offset = select i1 %pick, i64 0, i64 8
 %source = getelementptr i8, ptr %slots, i64 %offset
 {read}
 %got = load i8, ptr %pointer, align 1
 %expected = {expected}
 %ok = icmp eq i8 %got, %expected
 ret i1 %ok"
    )
}

#[test]
fn symbolic_pointer_loads_select_the_stored_slot() {
    let read = "%pointer = load ptr, ptr %source, align 8";
    let expected = "select i1 %pick, i8 7, i8 9";
    assert_eq!(verify(&fixture("", read, expected), 1000), "proved");
    assert_eq!(
        verify(&fixture("", read, "add i8 7, 0"), 1000),
        "counterexample"
    );
    assert_eq!(verify(&fixture("", read, expected), 1), "unknown");
}

#[test]
fn symbolic_whole_integer_copy_preserves_provenance() {
    let read = "%bits = load i64, ptr %source, align 8
 %copy = alloca ptr, align 8
 store i64 %bits, ptr %copy, align 8
 %pointer = load ptr, ptr %copy, align 8";
    assert_eq!(
        verify(&fixture("", read, "select i1 %pick, i8 7, i8 9"), 1000),
        "proved"
    );
    assert_eq!(
        verify(&fixture("", read, "add i8 0, 0"), 1000),
        "counterexample"
    );
}

#[test]
fn residual_slots_do_not_gain_provenance_from_equal_addresses() {
    let base = fixture(
        "",
        "%pointer = load ptr, ptr %source, align 8",
        "add i8 7, 0",
    )
    .replace("i64 0, i64 8", "i64 0, i64 16");
    // A computation drops integer-copy origin, even when it leaves the
    // address bits identical. Equality alone cannot recreate a pointer slot.
    let integer = base.replace(" %offset =", " %third = getelementptr i8, ptr %slots, i64 16\n %original = ptrtoint ptr %a to i64\n %address = or i64 %original, 0\n store i64 %address, ptr %third, align 8\n %offset =");
    assert_eq!(verify(&integer, 1000), "unknown");
    // The uncovered unwritten slot is undefined, and observing it fails.
    assert_eq!(verify(&base, 1000), "counterexample");
}

#[test]
fn selected_slots_keep_byte_integrity_poison_and_lifetimes() {
    let read = "%pointer = load ptr, ptr %source, align 8";
    let expected = "select i1 %pick, i8 7, i8 9";
    let corrupt = fixture("store i64 1, ptr %second, align 8", read, expected);
    assert_eq!(verify(&corrupt, 1000), "unknown");
    let partial = fixture("store i8 1, ptr %second, align 1", read, expected);
    assert_eq!(verify(&partial, 1000), "unknown");
    let poison = fixture("store ptr poison, ptr %second, align 8", read, expected);
    assert_eq!(verify(&poison, 1000), "counterexample");
    let dead = fixture(
        "call void @llvm.lifetime.end.p0(i64 1, ptr %b)",
        read,
        expected,
    );
    assert_eq!(verify(&dead, 1000), "counterexample");
    let outside = fixture("", read, expected).replace("i64 0, i64 8", "i64 0, i64 24");
    assert_eq!(verify(&outside, 1000), "counterexample");
}

#[test]
fn symbolic_pointer_stores_replace_only_the_selected_slot() {
    let text = fixture(
        "",
        "store ptr %a, ptr %source, align 8\n %pointer = load ptr, ptr %source, align 8",
        "add i8 7, 0",
    );
    assert_eq!(verify(&text, 1000), "proved");
    let other = fixture(
        "",
        "store ptr %a, ptr %source, align 8\n %pointer = load ptr, ptr %second, align 8",
        "select i1 %pick, i8 9, i8 7",
    );
    assert_eq!(verify(&other, 1000), "proved");
}

#[test]
fn whole_pointer_copies_support_symbolic_source_destination_and_length() {
    let copy = "%destination = alloca [16 x i8], align 8
 %to_offset = select i1 %other, i64 0, i64 8
 %to = getelementptr i8, ptr %destination, i64 %to_offset
 call void @llvm.memcpy.p0.p0.i64(ptr %to, ptr %source, i64 8, i1 false)
 %pointer = load ptr, ptr %to, align 8";
    let text = fixture("", copy, "select i1 %pick, i8 7, i8 9");
    assert_eq!(verify(&text, 1000), "proved");
    assert_eq!(
        verify(&text.replace("i8 7, i8 9", "i8 0, i8 0"), 1000),
        "counterexample"
    );
    // A partial copy cannot create a new whole pointer slot.
    assert_eq!(
        verify(&text.replace("i64 8, i1 false", "i64 7, i1 false"), 1000),
        "unknown"
    );

    let length = "%length = select i1 %pick, i64 0, i64 8
 call void @llvm.memmove.p0.p0.i64(ptr %slots, ptr %second, i64 %length, i1 false)
 %pointer = load ptr, ptr %slots, align 8";
    let text = fixture("", length, "select i1 %pick, i8 7, i8 9");
    assert_eq!(verify(&text, 1000), "proved");
    let partial = text.replace("i64 0, i64 8\n call void", "i64 7, i64 8\n call void");
    assert_eq!(verify(&partial, 1000), "unknown");
}

#[test]
fn symbolic_fill_invalidates_only_its_written_range() {
    let text = fixture(
        "",
        "call void @llvm.memset.p0.i64(ptr %source, i8 1, i64 8, i1 false)
 %offset_other = select i1 %pick, i64 8, i64 0
 %untouched = getelementptr i8, ptr %slots, i64 %offset_other
 %pointer = load ptr, ptr %untouched, align 8",
        "select i1 %pick, i8 9, i8 7",
    );
    assert_eq!(verify(&text, 1000), "proved");
    let corrupt = text.replace("ptr %untouched, align 8", "ptr %source, align 8");
    assert_eq!(verify(&corrupt, 1000), "unknown");
}

#[test]
fn realloc_preserves_only_whole_symbolic_slots_in_the_retained_prefix() {
    let text = fixture(
        "",
        "%heap = call ptr @__rust_alloc(i64 16, i64 8)
 %to = getelementptr i8, ptr %heap, i64 %offset
 store ptr %a, ptr %to, align 8
 %grown = call ptr @__rust_realloc(ptr %heap, i64 16, i64 8, i64 24)
 %from = getelementptr i8, ptr %grown, i64 %offset
 %pointer = load ptr, ptr %from, align 8",
        "add i8 7, 0",
    );
    assert_eq!(verify(&text, 1000), "proved");
    assert_eq!(
        verify(&text.replace("i64 8, i64 24", "i64 8, i64 8"), 1000),
        "counterexample"
    );
}
