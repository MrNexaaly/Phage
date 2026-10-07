//! Regressions from running Phage on repomap's readers (2026-10-01): a
//! literal-address phi incoming, a select over a partial undef choice, and
//! block visits counted per call. Each has a failing or unknown twin.

use super::Engine;
use crate::ir::Module;
use std::rc::Rc;

fn verify(text: &str, unwind: usize) -> String {
    let module = Module::new("user", text.to_owned(), false).expect("fixture parses");
    let entry = module.function("phage_target").expect("entry");
    let mut engine = Engine::new(5000, unwind, 1000).expect("Z3 required");
    engine.modules.push(Rc::new(module));
    engine
        .verify(&entry)
        .map(|v| v.status)
        .unwrap_or_else(|_| "unknown".into())
}
fn target(parameters: &str, blocks: &str) -> String {
    format!("define i1 @phage_target({parameters}) {{\n{blocks}\n}}\n")
}

/// `NonNull::dangling()` reaches a phi as `inttoptr (i64 1 to ptr)`; the
/// object must exist before the phi reads it.
#[test]
fn literal_address_phi_incoming() {
    let phi = |incoming: &str, tail: &str| {
        target(
            "i1 %c",
            &format!(
                "start:\n br i1 %c, label %a, label %b\na:\n br label %m\nb:\n br label %m\nm:\n %p = phi ptr [ inttoptr (i64 8 to ptr), %a ], [ {incoming}, %b ]\n{tail}"
            ),
        )
    };
    let address = " %i = ptrtoint ptr %p to i64\n";
    assert_eq!(
        verify(
            &phi(
                "null",
                &format!("{address} %v = icmp ule i64 %i, 8\n ret i1 %v")
            ),
            16
        ),
        "proved"
    );
    assert_eq!(
        verify(
            &phi(
                "null",
                &format!("{address} %v = icmp eq i64 %i, 8\n ret i1 %v")
            ),
            16
        ),
        "counterexample"
    );
    // A dangling pointer has no bytes to read.
    assert_eq!(
        verify(
            &phi(
                "inttoptr (i64 8 to ptr)",
                " %x = load i8, ptr %p, align 1\n ret i1 true"
            ),
            16
        ),
        "counterexample"
    );
}

/// Globals first named by a phi exist before it reads them (codex review).
#[test]
fn global_phi_incoming() {
    let text = |expected: u8| {
        format!(
            "@g = global i8 7, align 1\n{}",
            target(
                "",
                &format!(
                    "start:\n br label %join\njoin:\n %p = phi ptr [ @g, %start ]\n %x = load i8, ptr %p, align 1\n %ok = icmp eq i8 %x, {expected}\n ret i1 %ok"
                )
            )
        )
    };
    assert_eq!(verify(&text(7), 16), "proved");
    assert_eq!(verify(&text(8), 16), "counterexample");
}

/// An undef condition over two equal arms has no choice to make, also when
/// the arms are undef choices (codex review).
#[test]
fn undef_condition_over_equal_choices() {
    let text = |expected: u8| {
        target(
            "",
            &format!(
                "start:\n %a = select i1 false, i8 undef, i8 7\n %r = select i1 undef, i8 %a, i8 %a\n %ok = icmp eq i8 %r, {expected}\n ret i1 %ok"
            ),
        )
    };
    assert_eq!(verify(&text(7), 16), "proved");
    assert_eq!(verify(&text(8), 16), "counterexample");
}

/// A select whose arm is itself a select over undef.
#[test]
fn select_over_nested_undef_choices() {
    // %a is undef when `a` holds, else 7; %b is 9 unless `b` fails.
    let nested = |parameters: &str, a: &str, b: &str, outer: &str, tail: &str| {
        target(
            parameters,
            &format!(
                "start:\n br label %m\nm:\n %u = phi i8 [ undef, %start ]\n %a = select i1 {a}, i8 %u, i8 7\n %b = select i1 {b}, i8 9, i8 %u\n %r = select i1 {outer}\n{tail}"
            ),
        )
    };
    let one_of = " %x = icmp eq i8 %r, 7\n %y = icmp eq i8 %r, 9\n %v = or i1 %x, %y\n ret i1 %v";
    let check = |parameters: &str, a: &str, b: &str, outer: &str, tail: &str| {
        verify(&nested(parameters, a, b, outer, tail), 16)
    };
    // Neither inner select picks undef: 7 or 9.
    assert_eq!(
        check("i1 %d", "false", "true", "%d, i8 %a, i8 9", one_of),
        "proved"
    );
    assert_eq!(
        check("i1 %d", "false", "true", "%d, i8 %a, i8 %b", one_of),
        "proved"
    );
    // The undef side of the condition is the one that counts.
    assert_eq!(
        check("", "true", "true", "true, i8 %b, i8 %a", one_of),
        "proved"
    );
    assert_eq!(
        check("i1 %d", "true", "true", "%d, i8 %b, i8 %a", one_of),
        "counterexample"
    );
    assert_eq!(
        check("i1 %c", "%c", "true", "false, i8 %a, i8 %b", one_of),
        "proved"
    );
    // When the chosen arm picks undef, any value: not only 7 or 9...
    assert_eq!(
        check("i1 %c, i1 %d", "%c", "true", "%d, i8 %a, i8 9", one_of),
        "counterexample"
    );
    // ...yet defined bits stay defined (undef & 0 is 0).
    let masked = " %z = and i8 %r, 0\n %v = icmp eq i8 %z, 0\n ret i1 %v";
    assert_eq!(
        check("i1 %c, i1 %d", "%c", "%c", "%d, i8 %a, i8 %b", masked),
        "proved"
    );
    // A poison outer condition poisons the result.
    let stable = " %v = icmp eq i8 %r, %r\n ret i1 %v";
    assert_eq!(
        check("", "false", "true", "poison, i8 %a, i8 9", stable),
        "counterexample"
    );
    // Branching where the result may be undef is undefined behavior.
    let branch = |outer: &str| {
        target(
            "i1 %c, i1 %d",
            &format!(
                "start:\n br label %m\nm:\n %u = phi i1 [ undef, %start ]\n %a = select i1 %c, i1 %u, i1 true\n %r = select i1 {outer}, i1 %a, i1 false\n br i1 %r, label %e, label %e\ne:\n ret i1 true"
            ),
        )
    };
    assert_eq!(verify(&branch("%d"), 16), "counterexample");
    assert_eq!(verify(&branch("false"), 16), "proved");
}

/// Each call counts its own loop iterations; the caller's count survives
/// the call.
#[test]
fn block_visits_are_counted_per_call() {
    let count = "define i8 @count(i8 %n) {\nstart:\n br label %head\nhead:\n %i = phi i8 [ 0, %start ], [ %j, %body ]\n %done = icmp eq i8 %i, %n\n br i1 %done, label %exit, label %body\nbody:\n %j = add i8 %i, 1\n br label %head\nexit:\n ret i8 %i\n}\n";
    let calls = |n: u8, sum: u8| {
        format!(
            "{count}{}",
            target(
                "",
                &format!(
                    "start:\n %a = call i8 @count(i8 {n})\n %b = call i8 @count(i8 {n})\n %c = call i8 @count(i8 {n})\n %s = add i8 %a, %b\n %t = add i8 %s, %c\n %v = icmp eq i8 %t, {sum}\n ret i1 %v"
                )
            )
        )
    };
    // Three calls of 11 visits each fit a bound of 16.
    assert_eq!(verify(&calls(10, 30), 16), "proved");
    assert_eq!(verify(&calls(10, 31), 16), "counterexample");
    // One call longer than the bound is still a limit.
    assert_eq!(verify(&calls(20, 60), 16), "unknown");
    // A caller loop that calls on every turn keeps its own count.
    let caller = format!(
        "define i8 @id(i8 %x) {{\nstart:\n ret i8 %x\n}}\n{}",
        target(
            "",
            "start:\n br label %head\nhead:\n %i = phi i8 [ 0, %start ], [ %j, %body ]\n %d = icmp eq i8 %i, 20\n br i1 %d, label %exit, label %body\nbody:\n %k = call i8 @id(i8 %i)\n %j = add i8 %k, 1\n br label %head\nexit:\n ret i1 true"
        )
    );
    assert_eq!(verify(&caller, 16), "unknown");
    assert_eq!(verify(&caller, 32), "proved");
}

/// Two undef arms under a condition that may be poison: undef where the
/// condition is defined, poison where it is not (Phage on Phage found the
/// pattern in `u128::from_str_radix`).
#[test]
fn undef_arms_under_a_poison_condition() {
    let text = |guard: &str| {
        target(
            "i8 %x",
            &format!(
                "start:\n br label %m\nm:\n %u = phi i8 [ undef, %start ]\n {guard}\nbody:\n %y = add nuw i8 %x, 1\n %c = icmp eq i8 %y, 0\n %r = select i1 %c, i8 undef, i8 %u\n %z = and i8 %r, 0\n %ok = icmp eq i8 %z, 0\n ret i1 %ok\nout:\n ret i1 true"
            ),
        )
    };
    // Defined condition: undef & 0 is 0.
    let guarded = "%big = icmp eq i8 %x, -1\n br i1 %big, label %out, label %body";
    assert_eq!(verify(&text(guarded), 16), "proved");
    // x = 255 makes the condition poison, and so the result.
    assert_eq!(verify(&text("br label %body"), 16), "counterexample");
    // Below a byte an observed undef counts as poison: unknown, never a
    // false counterexample (codex review).
    let narrow = text(guarded)
        .replace("i8 undef, i8 %u", "i1 undef, i1 undef")
        .replace("and i8 %r, 0", "and i1 %r, false")
        .replace("icmp eq i8 %z, 0", "icmp eq i1 %z, false");
    assert_eq!(verify(&narrow, 16), "unknown");
}
