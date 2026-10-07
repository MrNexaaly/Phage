//! Optional source locations from LLVM debug metadata. Missing or malformed
//! metadata never affects execution semantics and never invents a location.

use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
pub struct DebugInfo {
    nodes: BTreeMap<usize, String>,
}

fn number(text: &str) -> Option<usize> {
    let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}
fn reference(text: &str, field: &str) -> Option<usize> {
    number(text.split_once(&format!("{field}: !"))?.1)
}
fn integer(text: &str, field: &str) -> Option<u32> {
    number(text.split_once(&format!("{field}: "))?.1)?
        .try_into()
        .ok()
}
fn quoted(text: &str, field: &str) -> Option<String> {
    let text = text.split_once(&format!("{field}: \""))?.1;
    let mut bytes = Vec::new();
    let mut input = text.bytes();
    while let Some(byte) = input.next() {
        if byte == b'"' {
            return String::from_utf8(bytes).ok();
        }
        if byte == b'\\' {
            let hi = (input.next()? as char).to_digit(16)?;
            let lo = (input.next()? as char).to_digit(16)?;
            bytes.push((hi * 16 + lo) as u8);
        } else {
            bytes.push(byte);
        }
    }
    None
}

impl DebugInfo {
    pub fn new(text: &str) -> Self {
        let nodes = text
            .lines()
            .filter_map(|line| {
                let (id, value) = line.strip_prefix('!')?.split_once(" = ")?;
                Some((id.parse().ok()?, value.into()))
            })
            .collect();
        Self { nodes }
    }
    pub fn location(&self, instruction: &str) -> Option<Location> {
        let id = number(instruction.split_once("!dbg !")?.1)?;
        let node = self.nodes.get(&id)?;
        if !node.starts_with("!DILocation(") {
            return None;
        }
        let line = integer(node, "line")?;
        if line == 0 {
            return None;
        }
        let column = integer(node, "column").unwrap_or(0);
        let mut scope = reference(node, "scope")?;
        for _ in 0..64 {
            let scope_node = self.nodes.get(&scope)?;
            if scope_node.starts_with("!DIFile(") {
                let filename = quoted(scope_node, "filename")?;
                let directory = quoted(scope_node, "directory")?;
                let path = PathBuf::from(directory).join(filename);
                let path = path.canonicalize().unwrap_or(path);
                return Some(Location {
                    file: path.to_string_lossy().into(),
                    line,
                    column,
                });
            }
            scope = reference(scope_node, "file").or_else(|| reference(scope_node, "scope"))?;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexical_scope_preserves_original_source() {
        let debug = DebugInfo::new(
            "!1 = !DIFile(filename: \"a\\20b.rs\", directory: \"/snapshot\")\n!2 = distinct !DISubprogram(file: !1)\n!3 = !DILexicalBlock(scope: !2)\n!4 = !DILocation(line: 7, column: 11, scope: !3, inlinedAt: !99)",
        );
        let location = debug.location("call void @panic(), !dbg !4").unwrap();
        assert_eq!(location.file, "/snapshot/a b.rs");
        assert_eq!((location.line, location.column), (7, 11));
        assert!(debug.location("ret i1 true").is_none());
    }
    #[test]
    fn missing_or_cyclic_metadata_has_no_fabricated_location() {
        let debug =
            DebugInfo::new("!1 = !DILexicalBlock(scope: !1)\n!2 = !DILocation(line: 2, scope: !1)");
        assert!(debug.location("ret i1 true, !dbg !2").is_none());
    }
}
