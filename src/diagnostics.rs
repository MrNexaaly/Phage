//! Stable diagnostic categories and original Rust locations. Unsupported
//! behavior stays unknown and is never disguised as an application failure.

use crate::{debug::Location, report::quote};
use std::collections::BTreeMap;

/// Attach the checked compiler's identity to model/signature failures, never
/// to solver uncertainty, resource limits or application counterexamples.
pub fn llvm_drift(message: &str, compiler: Option<&str>) -> String {
    let construct = message.contains("unrecognized instruction")
        || message.contains("unsupported intrinsic signature")
        || message.contains("unrecognized lifetime intrinsic")
        || message.contains("unmodeled call llvm.")
        || message.contains("llvm.") && message.contains("expects") && message.contains("operands");
    if !construct {
        return message.into();
    }
    let context = if let Some(compiler) = compiler {
        let release = crate::toolchain::field(compiler, "release:").unwrap_or("unknown".into());
        let llvm = crate::toolchain::field(compiler, "LLVM version:").unwrap_or("unknown".into());
        format!("LLVM IR from rustc {release} / LLVM {llvm} is newer than this construct's model")
    } else {
        "LLVM IR construct is outside this model (producer unknown for direct .ll input)".into()
    };
    format!("{message} | {context}")
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub llvm_line: Option<usize>,
    pub mir_line: Option<usize>,
    pub source: Option<Location>,
}

fn code(message: &str) -> &'static str {
    if message.contains("memory budget exceeded") {
        "memoryLimit"
    } else if message.contains("solver timeout after") {
        "solverTimeout"
    } else if message.contains("solver resource limit") {
        "solverResourceLimit"
    } else if message.contains("solver cancelled") {
        "solverCancelled"
    } else if message.contains("unmodeled call") {
        "unsupportedCall"
    } else if message.contains("unsupported entry argument")
        || message.contains("unsupported type")
        || message.contains("unsupported integer type")
    {
        "unsupportedType"
    } else if message.contains("unsupported instruction")
        || message.contains("unrecognized instruction")
    {
        "unsupportedInstruction"
    } else if message.contains("pointer choice is unresolved") {
        "unresolvedPointer"
    } else if message.contains("visit bound") {
        "blockLimit"
    } else if message.contains("state exploration limit") {
        "stateLimit"
    } else if message.contains("solver could not") || message.contains("solver unknown") {
        "solverUnknown"
    } else if message.contains("Z3 exited") || message.contains("Z3 closed") {
        "solverExit"
    } else if message.contains("Z3 returned") || message.contains("Broken pipe") {
        "solverError"
    } else if message.contains("rustc failed") {
        "compilerError"
    } else if message.contains("call depth limit") {
        "callDepthLimit"
    } else if message.contains("invalid bit pattern") {
        "invalidValue"
    } else if message.contains("MIR expired reference") {
        "invalidMemory"
    } else if message.contains("allocation bound")
        || message.contains("element bound")
        || message.contains("byte bound")
    {
        "allocationLimit"
    } else if message.contains("Rust assertion failed") {
        "assertionFailure"
    } else if message.contains("Rust panic call") {
        "rustPanic"
    } else if message.contains("property returned false") {
        "falseProperty"
    } else if message.contains("zero or poison divisor") {
        "invalidDivisor"
    } else if message.contains("invalid stack")
        || message.contains("invalid memory")
        || message.contains("invalid atomic memory")
        || message.contains("memory read")
        || message.contains("uninitialized")
    {
        "invalidMemory"
    } else if message.contains("attribute, which makes it poison") {
        "attributePoison"
    } else if message.contains("poison") {
        "poisonUse"
    } else if message.contains("unsupported") {
        "unsupportedFeature"
    } else {
        "analysis"
    }
}

pub fn explain(
    status: &str,
    message: &str,
    locations: &BTreeMap<usize, Location>,
) -> Vec<Diagnostic> {
    if status == "proved" {
        return Vec::new();
    }
    message
        .split("; ")
        .map(|message| {
            let llvm_line = message
                .rsplit_once("LLVM line ")
                .and_then(|(_, n)| n.split_whitespace().next()?.parse().ok());
            let mir_line = message
                .rsplit_once("MIR line ")
                .and_then(|(_, n)| n.split_whitespace().next()?.parse().ok());
            Diagnostic {
                code: code(message),
                message: message.into(),
                llvm_line,
                mir_line,
                source: llvm_line
                    .or(mir_line)
                    .and_then(|line| locations.get(&line).cloned()),
            }
        })
        .collect()
}

pub fn json(diagnostic: &Diagnostic) -> String {
    let llvm = diagnostic
        .llvm_line
        .map_or("null".into(), |l| l.to_string());
    let mir = diagnostic.mir_line.map_or("null".into(), |l| l.to_string());
    let source = diagnostic.source.as_ref().map_or("null".into(), |s| {
        format!(
            "{{\"file\":{},\"line\":{},\"column\":{}}}",
            quote(&s.file),
            s.line,
            s.column
        )
    });
    format!(
        "{{\"code\":{},\"message\":{},\"llvmLine\":{llvm},\"mirLine\":{mir},\"source\":{source}}}",
        quote(diagnostic.code),
        quote(&diagnostic.message)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_functions_limits_and_solver_errors_remain_distinct() {
        let diagnostics = explain(
            "unknown",
            "unmodeled call external_result at LLVM line 12; reachable block loop exceeds visit bound 2; Z3 exited unexpectedly",
            &BTreeMap::new(),
        );
        assert_eq!(
            diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
            ["unsupportedCall", "blockLimit", "solverExit"]
        );
        assert_eq!(diagnostics[0].llvm_line, Some(12));
        assert!(diagnostics[0].source.is_none());
        assert!(explain("proved", "all paths completed", &BTreeMap::new()).is_empty());
    }

    #[test]
    fn model_drift_names_compiler_without_reclassifying_other_unknowns() {
        let compiler = Some("release: 1.99.0\nLLVM version: 23.1.1\n");
        for message in [
            "unrecognized instruction future_op",
            "unsupported intrinsic signature llvm.lifetime.start.p0",
            "llvm.ctpop.i8 expects 1 operands",
            "unmodeled call llvm.future.i8",
        ] {
            let detail = llvm_drift(message, compiler);
            assert!(detail.contains(
                "LLVM IR from rustc 1.99.0 / LLVM 23.1.1 is newer than this construct's model"
            ));
            assert!(detail.starts_with(message));
        }
        for message in [
            "solver unknown",
            "state exploration limit reached",
            "unmodeled call external",
            "property returned false",
            "symbolic allocation size is unsupported",
            "partial lifetimes are unsupported",
            "unsupported instruction freeze",
            "frem (fmod) is unsupported",
            "unsupported pointer vector lanes",
        ] {
            assert_eq!(llvm_drift(message, compiler), message);
        }
        assert!(
            llvm_drift("unrecognized instruction future_op", None).contains("producer unknown")
        );
    }
}
