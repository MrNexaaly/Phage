//! Symbol and syntax helpers: v0 Rust mangling paths (allocator entry
//! points) and balanced-parenthesis scanning of LLVM operand lists.

/// Recognize only core/std panic namespaces, in legacy and v0 mangling.
pub(crate) fn is_panic(symbol: &str) -> bool {
    symbol.starts_with("_ZN4core9panicking")
        || symbol.starts_with("_ZN3std9panicking")
        || v0_segments(symbol)
            .is_some_and(|p| p.len() >= 3 && matches!(p[0], "core" | "std") && p[1] == "panicking")
}

/// Identifiers of a v0 symbol whose path is a crate followed by nested
/// value/type namespaces (`_RNvNtCs1_4core6option13unwrap_failed`). Generic,
/// impl and back-referenced paths return `None`.
pub(crate) fn v0_segments(symbol: &str) -> Option<Vec<&str>> {
    let mut rest = symbol
        .strip_prefix("_R")?
        .trim_start_matches(|c: char| c.is_ascii_digit());
    let mut depth = 0;
    while let Some(inner) = rest.strip_prefix('N') {
        rest = inner.get(1..)?;
        depth += 1;
    }
    rest = rest.strip_prefix('C')?;
    let mut segments = Vec::new();
    for _ in 0..=depth {
        if let Some(disambiguated) = rest.strip_prefix('s') {
            rest = &disambiguated[disambiguated.find('_')? + 1..];
        }
        let digits = rest.find(|c: char| !c.is_ascii_digit())?;
        let length: usize = rest[..digits].parse().ok()?;
        rest = &rest[digits..];
        // A `_` separates the length from identifiers starting with a digit or `_`.
        rest = rest
            .strip_prefix('_')
            .filter(|_| length > 0)
            .unwrap_or(rest);
        segments.push(rest.get(..length)?);
        rest = &rest[length..];
    }
    rest.is_empty().then_some(segments)
}

/// Index of the parenthesis closing the one at `open`, ignoring quoted text.
pub(crate) fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    for (i, byte) in text.bytes().enumerate().skip(open) {
        match byte {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panic_namespace_matches_legacy_and_v0_but_not_lookalikes() {
        for symbol in [
            "_ZN4core9panicking5panic17habcE",
            "_ZN3std9panicking11begin_panic17habcE",
            "_RNvNtCs123_4core9panicking5panic",
            "_RNvNtCs123_3std9panicking11begin_panic",
        ] {
            assert!(is_panic(symbol), "{symbol}");
        }
        for symbol in [
            "_RNvNtCs123_3app9panicking5panic",
            "_RNvNtCs123_4core9panicking",
            "_RNvNtCs123_4core10panickingx5panic",
            "panic",
            "_Rmalformed",
        ] {
            assert!(!is_panic(symbol), "{symbol}");
        }
    }
}
