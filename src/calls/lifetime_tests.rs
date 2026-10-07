//! Old/new whole-object lifetime forms, restart definedness and refusals.
use crate::{
    engine::{Engine, Verdict},
    ir::Module,
};
use std::rc::Rc;

fn check(body: &str) -> Result<Verdict, String> {
    let text =
        format!("define i1 @phage_target() {{\nstart:\n %a = alloca i8, align 1\n{body}\n}}\n");
    let module = Module::new("user", text, false).unwrap();
    let entry = module.function("phage_target").unwrap();
    let mut engine = Engine::new(5000, 16, 1000).unwrap();
    engine.modules.push(Rc::new(module));
    engine.verify(&entry)
}

fn status(body: &str) -> String {
    check(body)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}

#[test]
fn both_lifetime_forms_have_identical_whole_object_semantics() {
    for prefix in ["i64 1, ", "i64 -1, ", ""] {
        let start = format!(" call void @llvm.lifetime.start.p0({prefix}ptr %a)");
        let end = format!(" call void @llvm.lifetime.end.p0({prefix}ptr %a)");
        let write_read = " store i8 7, ptr %a, align 1\n %v = load i8, ptr %a, align 1\n %ok = icmp eq i8 %v, 7\n ret i1 %ok";
        assert_eq!(status(&format!("{start}\n{write_read}")), "proved");
        assert_eq!(status(&format!("{end}\n{start}\n{write_read}")), "proved");
        assert_eq!(
            status(&format!(
                "{start}\n store i8 7, ptr %a, align 1\n{end}\n %v = load i8, ptr %a, align 1\n ret i1 true"
            )),
            "proved"
        );
        assert_eq!(
            status(&format!(
                "{start}\n store i8 7, ptr %a, align 1\n{end}\n{start}\n %v = load i8, ptr %a, align 1\n %ok = icmp eq i8 %v, 7\n ret i1 %ok"
            )),
            "counterexample"
        );
    }
}

#[test]
fn partial_nonbase_and_unrecognized_lifetimes_stay_unknown() {
    for body in [
        " call void @llvm.lifetime.start.p0(i64 0, ptr %a)\n ret i1 true",
        " %p = getelementptr i8, ptr %a, i64 1\n call void @llvm.lifetime.end.p0(i64 1, ptr %p)\n ret i1 true",
        " %p = getelementptr i8, ptr %a, i64 1\n call void @llvm.lifetime.end.p0(ptr %p)\n ret i1 true",
        " call void @llvm.lifetime.start.p0()\n ret i1 true",
        " call void @llvm.lifetime.start.p0(ptr %a, ptr %a, ptr %a)\n ret i1 true",
        " call void @llvm.lifetime.future.p0(ptr %a)\n ret i1 true",
        " call void @llvm.lifetime.start.p0(i64 1)\n ret i1 true",
    ] {
        assert_eq!(status(body), "unknown", "{body}");
    }
}
