//! Positive, negative and refusal controls over valid direct LLVM.
use crate::{engine::Engine, ir::Module};
use std::rc::Rc;
fn run(params: &str, body: &str, bound: Option<u64>) -> Result<crate::engine::Verdict, String> {
    let module = Module::new(
        "user",
        format!("define i1 @phage_target({params}) {{\nstart:\n{body}\n}}\n"),
        true,
    )?;
    let target = module.function("phage_target")?;
    let mut engine = Engine::new(5000, 16, 1000)?;
    engine.max_slice_len = bound;
    engine.modules.push(Rc::new(module));
    engine.verify(&target)
}
const ARRAY: &str = "ptr noalias noundef readonly align 1 dereferenceable(4) %bytes";
const SLICE: &str = "ptr noalias noundef nonnull readonly align 1 %bytes.0, i64 noundef %bytes.1";
#[test]
fn bounded_parser_and_byte_witness() {
    assert_eq!(
        run(
            ARRAY,
            "%x = load i8, ptr %bytes, align 1\n%c = icmp eq i8 %x, %x\nret i1 %c",
            None
        )
        .unwrap()
        .status,
        "proved"
    );
    let bad = run(
        ARRAY,
        "%x = load i8, ptr %bytes, align 1\n%c = icmp eq i8 %x, 7\nret i1 %c",
        None,
    )
    .unwrap();
    assert_eq!(bad.status, "counterexample");
    assert!(bad.model.contains("witness_input_bytes0_0"));
    assert!(bad.model.contains("bytes[3]"));
}
#[test]
fn array_one_past_read_fails() {
    assert_eq!(
        run(
            ARRAY,
            "%p = getelementptr i8, ptr %bytes, i64 4\n%x = load i8, ptr %p, align 1\nret i1 true",
            None
        )
        .unwrap()
        .status,
        "counterexample"
    );
}
#[test]
fn slice_exact_length_is_access_bound() {
    assert_eq!(run(SLICE, "%p = getelementptr i8, ptr %bytes.0, i64 %bytes.1\n%x = load i8, ptr %p, align 1\nret i1 true", Some(4)).unwrap().status,"counterexample");
    assert_eq!(run(SLICE, "%c = icmp eq i64 %bytes.1, 0\nbr i1 %c, label %done, label %read\nread:\n%x = load i8, ptr %bytes.0, align 1\n%v = icmp eq i8 %x, %x\nret i1 %v\ndone:\nret i1 true",Some(4)).unwrap().status,"proved");
    assert!(
        run(SLICE, "ret i1 true", None)
            .unwrap_err()
            .contains("--maxSliceLen")
    );
}
#[test]
fn readonly_scalar_and_intrinsic_writes_fail() {
    for body in [
        "store i8 0, ptr %bytes, align 1\nret i1 true",
        "call void @llvm.memset.p0.i64(ptr %bytes, i8 0, i64 1, i1 false)\nret i1 true",
        "call void @llvm.memcpy.p0.p0.i64(ptr %bytes, ptr %bytes, i64 1, i1 false)\nret i1 true",
    ] {
        assert_eq!(run(ARRAY, body, None).unwrap().status, "counterexample");
    }
}
#[test]
fn mutable_inputs_are_initialized_and_writable() {
    let mutable = ARRAY.replace("readonly ", "");
    assert_eq!(run(&mutable,"%x = load i8, ptr %bytes, align 1\nstore i8 %x, ptr %bytes, align 1\n%y = load i8, ptr %bytes, align 1\n%c = icmp eq i8 %x, %y\nret i1 %c",None).unwrap().status,"proved");
}
#[test]
fn reference_objects_never_alias_each_other_or_constants() {
    let params = format!("{ARRAY}, ptr noalias noundef readonly dereferenceable(4) %other");
    assert_eq!(
        run(&params, "%c = icmp ne ptr %bytes, %other\nret i1 %c", None)
            .unwrap()
            .status,
        "proved"
    );
}
#[test]
fn arbitrary_pointers_and_oversized_buffers_stay_unknown() {
    for params in [
        "ptr %p",
        "ptr noalias noundef nonnull %p",
        "ptr noalias noundef readonly dereferenceable(1048577) %p",
        "ptr noalias noundef byval(i8) dereferenceable(4) %p",
    ] {
        assert!(run(params, "ret i1 true", Some(4)).is_err(), "{params}");
    }
}
#[test]
fn page_sized_symbolic_offset_read_proves() {
    let params = "ptr noalias noundef readonly dereferenceable(4096) %bytes, i16 %index";
    let body = "%i = zext i16 %index to i64\n%in = icmp ult i64 %i, 4096\nbr i1 %in, label %read, label %done\nread:\n%p = getelementptr inbounds i8, ptr %bytes, i64 %i\n%x = load i8, ptr %p, align 1\n%c = icmp eq i8 %x, %x\nret i1 %c\ndone:\nret i1 true";
    assert_eq!(run(params, body, None).unwrap().status, "proved");
}
#[test]
fn declared_alignment_is_preserved_and_overstated_access_fails() {
    assert_eq!(
        run(
            ARRAY,
            "%x = load i32, ptr %bytes, align 4\nret i1 true",
            None
        )
        .unwrap()
        .status,
        "counterexample"
    );
    assert_eq!(
        run(
            &ARRAY.replace("align 1", "align 4"),
            "%x = load i32, ptr %bytes, align 4\n%c = icmp eq i32 %x, %x\nret i1 %c",
            None
        )
        .unwrap()
        .status,
        "proved"
    );
}
#[test]
fn zero_length_readonly_intrinsics_remain_noops() {
    assert_eq!(
        run(
            ARRAY,
            "call void @llvm.memset.p0.i64(ptr %bytes, i8 0, i64 0, i1 false)\nret i1 true",
            None
        )
        .unwrap()
        .status,
        "proved"
    );
}
#[test]
fn writeonly_reads_fail_and_neutral_dereferenceability_is_allowed() {
    let params = ARRAY.replace("readonly", "writeonly");
    assert_eq!(
        run(
            &params,
            "%x = load i8, ptr %bytes, align 1\nret i1 true",
            None
        )
        .unwrap()
        .status,
        "counterexample"
    );
    assert_eq!(
        run(
            &params,
            "%x = call i32 @memcmp(ptr %bytes, ptr %bytes, i64 1)\nret i1 true",
            None
        )
        .unwrap()
        .status,
        "counterexample"
    );
}
#[test]
fn readonly_entry_is_disjoint_from_global_constant() {
    let text = format!(
        "@constant = private constant [4 x i8] zeroinitializer, align 1\ndefine i1 @phage_target({ARRAY}) {{\nstart:\n%c = icmp ne ptr %bytes, @constant\nret i1 %c\n}}\n"
    );
    let module = Module::new("user", text, true).unwrap();
    let target = module.function("phage_target").unwrap();
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    engine.modules.push(Rc::new(module));
    assert_eq!(engine.verify(&target).unwrap().status, "proved");
}
#[test]
fn pointer_address_select_preserves_only_known_provenance() {
    let params =
        format!("{ARRAY}, ptr noalias noundef readonly dereferenceable(4) %other, i1 %which");
    let body = "%a = ptrtoint ptr %bytes to i64\n%b = ptrtoint ptr %other to i64\n%s = select i1 %which, i64 %a, i64 %b\n%p = inttoptr i64 %s to ptr\n%x = load i8, ptr %p, align 1\n%c = icmp eq i8 %x, %x\nret i1 %c";
    assert_eq!(run(&params, body, None).unwrap().status, "proved");
    let bad = body.replace(
        "%x = load i8, ptr %p",
        "%end = getelementptr i8, ptr %p, i64 4\n%x = load i8, ptr %end",
    );
    assert_eq!(run(&params, &bad, None).unwrap().status, "counterexample");
    let fabricated = body.replace("i64 %b\n%p", "i64 65536\n%p");
    let result = run(&params, &fabricated, None);
    assert!(result.is_err() || result.unwrap().status == "unknown");
}
#[test]
fn whole_object_copy_keeps_bytes_and_bounds() {
    let params = "ptr noalias noundef readonly dereferenceable(4096) %source, ptr noalias noundef dereferenceable(4096) %target, i16 %index";
    let body = "call void @llvm.memcpy.p0.p0.i64(ptr %target, ptr %source, i64 4096, i1 false)\n%i = zext i16 %index to i64\n%ok = icmp ult i64 %i, 4096\nbr i1 %ok, label %read, label %done\nread:\n%s = getelementptr i8, ptr %source, i64 %i\n%t = getelementptr i8, ptr %target, i64 %i\n%a = load i8, ptr %s, align 1\n%b = load i8, ptr %t, align 1\n%c = icmp eq i8 %a, %b\nret i1 %c\ndone:\nret i1 true";
    assert_eq!(run(params, body, None).unwrap().status, "proved");
    assert_eq!(
        run(
            params,
            &body.replace("i64 4096, i1 false", "i64 4097, i1 false"),
            None
        )
        .unwrap()
        .status,
        "counterexample"
    );
}
#[test]
fn unnamed_slice_pairs_need_rustc_length_range() {
    let unnamed = "ptr noalias noundef nonnull readonly align 1 %0, i64 noundef range(i64 0, -9223372036854775808) %1";
    let read = "%c = icmp eq i64 %1, 0\nbr i1 %c, label %done, label %read\nread:\n%x = load i8, ptr %0, align 1\n%v = icmp eq i8 %x, %x\nret i1 %v\ndone:\nret i1 true";
    assert_eq!(run(unnamed, read, Some(4)).unwrap().status, "proved");
    let past = "%p = getelementptr i8, ptr %0, i64 %1\n%x = load i8, ptr %p, align 1\nret i1 true";
    assert_eq!(
        run(unnamed, past, Some(4)).unwrap().status,
        "counterexample"
    );
    // Without the slice-length range an unnamed pointer/i64 pair stays unsupported.
    let plain = "ptr noalias noundef nonnull readonly align 1 %0, i64 noundef %1";
    assert!(run(plain, "ret i1 true", Some(4)).is_err());
}
