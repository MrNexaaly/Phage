//! Backtracking controls: branch facts must be removed, and declarations
//! created on a branch must remain available after returning to its sibling.

use super::{Sat, Solver};

#[test]
fn sibling_paths_do_not_leak_assertions() {
    let mut solver = Solver::new(5000).unwrap();
    solver.declare("x", "(_ BitVec 8)").unwrap();
    let a = vec!["(= x (_ bv1 8))".into()];
    let b = vec!["(= x (_ bv2 8))".into()];
    assert_eq!(solver.check(&a, "true").unwrap(), Sat::Yes);
    assert_eq!(solver.check(&a, "(= x (_ bv2 8))").unwrap(), Sat::No);
    assert_eq!(solver.check(&b, "true").unwrap(), Sat::Yes);
    assert_eq!(solver.check(&[], "(= x (_ bv3 8))").unwrap(), Sat::Yes);
}

#[test]
fn fresh_symbols_survive_pop_and_model_recheck() {
    let mut solver = Solver::new(5000).unwrap();
    solver.declare("x", "(_ BitVec 8)").unwrap();
    solver.check(&["(= x (_ bv1 8))".into()], "true").unwrap();
    solver.declare("y", "(_ BitVec 8)").unwrap();
    let path = vec!["(= y (_ bv2 8))".into()];
    assert_eq!(solver.check(&path, "true").unwrap(), Sat::Yes);
    assert!(
        solver
            .model(&path, "true", &["y".into()])
            .unwrap()
            .contains("#x02")
    );
    assert_eq!(solver.check(&[], "(= y (_ bv3 8))").unwrap(), Sat::Yes);
}

#[test]
fn a_reused_prefix_is_asserted_once() {
    let mut solver = Solver::new(5000).unwrap();
    solver.declare("x", "(_ BitVec 8)").unwrap();
    let path = vec!["(= x (_ bv1 8))".into()];
    solver.check(&path, "true").unwrap();
    solver.check(&path, "true").unwrap();
    assert_eq!(solver.assertions, 3); // One path fact, two query obligations.
}

#[test]
fn killed_solver_is_an_error_with_signal_information_not_a_panic() {
    let mut solver = Solver::new(5000).unwrap();
    solver.child.kill().unwrap();
    solver.child.wait().unwrap();
    let error = solver.line().unwrap_err();
    assert!(error.contains("Z3 exited"));
    assert!(error.contains("signal"), "{error}");
}

#[test]
fn longest_query_keeps_its_context_and_standalone_obligation() {
    let mut solver = Solver::new(5000).unwrap();
    solver.declare("x", "(_ BitVec 8)").unwrap();
    solver.context = "instruction at LLVM line 19".into();
    solver
        .check(&["(= x (_ bv1 8))".into()], "(= x (_ bv2 8))")
        .unwrap();
    assert_eq!(solver.slowest_context, "instruction at LLVM line 19");
    assert!(solver.slowest_seconds > 0.0);
    assert!(solver.slowest_query.contains("(declare-fun x"));
    assert!(solver.slowest_query.contains("(assert (= x (_ bv2 8)))"));
}

#[test]
fn inbounds_offset_no_wrap_implies_absolute_address_no_wrap() {
    let mut solver = Solver::new(15000).unwrap();
    for name in ["base", "size", "offset", "index"] {
        solver.declare(name, "(_ BitVec 64)").unwrap();
    }
    let constraints = [
        "(not (= base (_ bv0 64)))",
        "(bvule base (bvsub (_ bv18446744073709551615 64) size))",
        "(bvule offset size)",
        "(bvule (bvadd offset index) size)",
        "(bvuge (bvadd offset index) offset)",
    ]
    .map(str::to_owned);
    let violation = "(not (bvuge (bvadd (bvadd base offset) index) (bvadd base offset)))";
    assert_eq!(solver.check(&constraints, violation).unwrap(), Sat::No);
}

/// A cancelled check keeps the process, its declarations and the asserted
/// prefix; SIGINT between commands ends Z3 and falls back to a restart.
#[test]
fn interrupted_check_resumes_in_place() {
    let mut solver = Solver::new(5000).unwrap();
    solver.declare("x", "(_ BitVec 8)").unwrap();
    let path = vec!["(= x (_ bv1 8))".to_string()];
    assert_eq!(solver.check(&path, "true").unwrap(), Sat::Yes);
    // Pigeonhole with 17 pigeons in 16 holes: far beyond the wait below.
    let holes = 16;
    for i in 0..=holes {
        solver.declare(&format!("p{i}"), "(_ BitVec 8)").unwrap();
    }
    let pigeons: Vec<String> = (0..=holes).map(|i| format!("p{i}")).collect();
    let mut hard = vec![format!("(distinct {})", pigeons.join(" "))];
    hard.extend(
        pigeons
            .iter()
            .map(|p| format!("(bvult {p} (_ bv{holes} 8))")),
    );
    solver.activate(&path).unwrap();
    solver.write("(push 1)").unwrap();
    solver
        .write(&format!("(assert (and {}))", hard.join(" ")))
        .unwrap();
    solver.write("(check-sat)").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    solver.interrupt().unwrap();
    assert_eq!((solver.interrupts, solver.restarts), (1, 0));
    // Same process: the prefix is still asserted, the cancelled fact is gone.
    assert_eq!(solver.check(&path, "(= x (_ bv2 8))").unwrap(), Sat::No);
    assert_eq!(solver.check(&path, "(= p0 (_ bv200 8))").unwrap(), Sat::Yes);
    assert_eq!(solver.restarts, 0);
    // Negative control: nothing running, so SIGINT ends Z3 and it restarts.
    solver.write("(push 1)").unwrap();
    solver.interrupt().unwrap();
    assert_eq!(solver.restarts, 1);
    assert_eq!(solver.check(&path, "(= x (_ bv1 8))").unwrap(), Sat::Yes);
}

#[test]
fn timeout_is_named_and_following_queries_are_not_cancelled() {
    let mut solver = Solver::new(1).unwrap();
    solver.portfolio = false;
    let pigeons: Vec<_> = (0..=16).map(|i| format!("p{i}")).collect();
    for p in &pigeons {
        solver.declare(p, "(_ BitVec 8)").unwrap();
    }
    let mut hard = vec![format!("(distinct {})", pigeons.join(" "))];
    hard.extend(pigeons.iter().map(|p| format!("(bvult {p} (_ bv16 8))")));
    assert_eq!(
        solver
            .check(&[], &format!("(and {})", hard.join(" ")))
            .unwrap(),
        Sat::Unknown
    );
    assert!(
        solver
            .last_unknown
            .contains("solver timeout after 1 ms on query 1"),
        "{}",
        solver.last_unknown
    );
    // Keep a generous follow-up timeout: scheduler/transport startup at
    // 1 ms is itself a valid timeout, not evidence of context corruption.
    solver.timeout_ms = 5000;
    solver.write("(set-option :timeout 5000)").unwrap();
    assert_eq!(solver.check(&[], "(= p0 (_ bv200 8))").unwrap(), Sat::Yes);
    assert_eq!(
        solver
            .check(&["(= p0 (_ bv200 8))".into()], "(= p0 (_ bv201 8))")
            .unwrap(),
        Sat::No
    );
}

#[test]
fn resource_cancellation_is_distinct_and_does_not_poison_context() {
    let mut solver = Solver::new(5000).unwrap();
    solver.portfolio = false;
    solver.declare("x", "(_ BitVec 8)").unwrap();
    solver.write("(set-option :rlimit 1)").unwrap();
    assert_eq!(solver.check(&[], "(= x (_ bv1 8))").unwrap(), Sat::Unknown);
    assert!(
        solver.last_unknown.contains("solver cancelled"),
        "{}",
        solver.last_unknown
    );
    assert_eq!(solver.check(&[], "(= x (_ bv2 8))").unwrap(), Sat::Yes);
}

#[test]
fn cancelled_push_is_unknown_and_restarted_before_the_next_query() {
    let mut solver = Solver::new(5000).unwrap();
    solver.portfolio = false;
    solver
        .declare("a", "(Array (_ BitVec 8) (_ BitVec 8))")
        .unwrap();
    // Real Z3 resource exhaustion in its SMT context, before check-sat.
    solver.write("(set-option :rlimit 1)").unwrap();
    assert_eq!(
        solver
            .check(&[], "(= (select a (_ bv0 8)) (_ bv1 8))")
            .unwrap(),
        Sat::Unknown
    );
    assert!(
        solver
            .last_unknown
            .contains("solver cancelled during context update on query 1"),
        "{}",
        solver.last_unknown
    );
    assert_eq!(solver.restarts, 1);
    assert_eq!(
        solver
            .check(&[], "(= (select a (_ bv0 8)) (_ bv2 8))")
            .unwrap(),
        Sat::Yes
    );
    assert_eq!(
        solver
            .check(
                &["(= (select a (_ bv0 8)) (_ bv2 8))".into()],
                "(= (select a (_ bv0 8)) (_ bv3 8))"
            )
            .unwrap(),
        Sat::No
    );
}
