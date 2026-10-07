//! Restricted parser for pinned built-MIR dumps. Statements remain unoptimized
//! and retain source spans. Unrecognized executable syntax is not discarded.

use super::registry::Registry;
use crate::{debug::Location, ir::Line};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Function {
    pub key: String,
    pub name: String,
    pub params: Vec<(String, String)>,
    pub result: String,
    pub locals: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub blocks: BTreeMap<String, Vec<Line>>,
    pub storage: BTreeSet<String>,
}
pub struct Program {
    pub functions: BTreeMap<String, Function>,
    pub locations: BTreeMap<usize, Location>,
    pub registry: Registry,
}

pub fn split(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut quote = false;
    let mut escape = false;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        if quote {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                quote = false;
            }
            continue;
        }
        match c {
            '"' => quote = true,
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1
            }
            _ => {}
        }
    }
    out.push(text[start..].trim());
    out
}
pub fn closing(text: &str, opening: usize) -> Option<usize> {
    let mut depth = 0;
    let mut quote = false;
    let mut escape = false;
    for (i, c) in text.char_indices().skip_while(|(i, _)| *i < opening) {
        if quote {
            if escape {
                escape = false
            } else if c == '\\' {
                escape = true
            } else if c == '"' {
                quote = false
            }
            continue;
        }
        match c {
            '"' => quote = true,
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => {
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
pub fn parentheses(mut text: &str) -> &str {
    while text.starts_with('(') && closing(text, 0) == Some(text.len() - 1) {
        text = text[1..text.len() - 1].trim();
    }
    text
}
pub fn base_type(text: &str) -> String {
    text.split('<')
        .next()
        .unwrap_or(text)
        .trim_end_matches("::")
        .to_owned()
}

impl Program {
    pub fn new(bundle: &str, expanded: &str) -> Result<Self, String> {
        let mut registry = Registry::new(expanded)?;
        if registry
            .definitions
            .keys()
            .any(|key| super::integer(key.rsplit("::").next().unwrap_or(key)).is_some())
        {
            return Err("unsupported MIR source type that shadows a primitive".into());
        }
        let mut functions = BTreeMap::new();
        let mut locations = BTreeMap::new();
        let lines: Vec<_> = bundle.lines().collect();
        let mut pos = 0;
        let mut key = String::new();
        while pos < lines.len() {
            let line = lines[pos];
            if let Some(k) = line.strip_prefix("// phage-function: ") {
                key = k.into();
            }
            if !line.starts_with("fn ") {
                pos += 1;
                continue;
            }
            let header = line
                .strip_prefix("fn ")
                .ok_or("MIR function header missing")?;
            let open = header.find('(').ok_or("MIR arguments missing")?;
            let close = closing(header, open).ok_or("MIR arguments unclosed")?;
            let name = header[..open].to_owned();
            let result = header[close + 1..]
                .trim()
                .strip_prefix("-> ")
                .and_then(|s| s.strip_suffix(" {"))
                .ok_or("MIR return type missing")?
                .to_owned();
            let mut params = Vec::new();
            let mut locals = BTreeMap::new();
            locals.insert("_0".into(), result.clone());
            for p in split(&header[open + 1..close]) {
                if p.is_empty() {
                    continue;
                }
                let (n, t) = p.split_once(": ").ok_or("MIR parameter type missing")?;
                params.push((n.into(), t.into()));
                locals.insert(n.into(), t.into());
            }
            let mut labels = BTreeMap::new();
            let mut blocks: BTreeMap<String, Vec<Line>> = BTreeMap::new();
            let mut block = None;
            let mut storage = BTreeSet::new();
            pos += 1;
            while pos < lines.len() && lines[pos] != "}" {
                let raw = lines[pos];
                let code = code(raw).trim();
                if let Some(local) = code.strip_prefix("let ") {
                    if let Some((n, t)) = local.trim_start_matches("mut ").split_once(": ") {
                        locals.insert(
                            n.into(),
                            t.trim_end_matches(';')
                                .split(" as UserTypeProjection")
                                .next()
                                .unwrap_or(t)
                                .into(),
                        );
                    }
                } else if let Some(debug) = code.strip_prefix("debug ") {
                    if let Some((name, local)) = debug.trim_end_matches(';').split_once(" => ") {
                        labels.insert(local.into(), name.into());
                    }
                } else if code.starts_with("bb") && code.ends_with(": {") {
                    let label = code
                        .split(':')
                        .next()
                        .ok_or("MIR block missing")?
                        .split_whitespace()
                        .next()
                        .ok_or("MIR block missing")?
                        .to_owned();
                    blocks.insert(label.clone(), Vec::new());
                    block = Some(label);
                } else if code == "}" {
                    block = None;
                } else if let Some(label) = &block
                    && !code.is_empty()
                {
                    if !code.ends_with(';') {
                        return Err(format!("unsupported MIR statement layout: {code}"));
                    }
                    if let Some(n) = code
                        .strip_prefix("StorageLive(")
                        .and_then(|s| s.strip_suffix(");"))
                    {
                        storage.insert(n.into());
                    }
                    if let Some(span) = span(raw) {
                        locations.insert(pos + 1, span);
                    }
                    blocks
                        .get_mut(label)
                        .ok_or("MIR block missing")?
                        .push(Line {
                            text: code.trim_end_matches(';').into(),
                            number: pos + 1,
                            noundef: false,
                        });
                }
                pos += 1;
            }
            let function_key = if key.is_empty() {
                name.clone()
            } else {
                key.clone()
            };
            if functions
                .insert(
                    function_key.clone(),
                    Function {
                        key: function_key,
                        name,
                        params,
                        result,
                        locals,
                        labels,
                        blocks,
                        storage,
                    },
                )
                .is_some()
            {
                return Err("duplicate MIR function key".into());
            }
            key.clear();
            pos += 1;
        }
        for function in functions.values() {
            if function.name.rsplit("::").next() == Some("drop") && function.params.len() == 1 {
                let ty = function.params[0]
                    .1
                    .trim_start_matches("&mut ")
                    .trim_start_matches('&');
                registry.destructors.insert(base_type(ty));
            }
        }
        Ok(Self {
            functions,
            locations,
            registry,
        })
    }
    pub fn entry(&self, name: &str) -> Result<&Function, String> {
        if let Some(f) = self.functions.get(name) {
            return Ok(f);
        }
        let candidates: Vec<_> = self.functions.values().filter(|f| f.name == name).collect();
        if candidates.len() == 1 {
            Ok(candidates[0])
        } else {
            Err(format!("MIR entry {name} missing or ambiguous"))
        }
    }
}
fn span(raw: &str) -> Option<Location> {
    let text = raw.rsplit_once(" at ")?.1;
    let mut pieces = text.rsplitn(5, ':');
    pieces.next()?;
    pieces.next()?;
    let column = pieces.next()?.parse().ok()?;
    let line = pieces.next()?.parse().ok()?;
    let file = pieces.next()?;
    Some(Location {
        file: std::path::Path::new(file)
            .canonicalize()
            .unwrap_or_else(|_| file.into())
            .to_string_lossy()
            .into(),
        line,
        column,
    })
}

// Comment markers in string constants are data, including URLs.
fn code(line: &str) -> &str {
    let mut quote = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if quote {
            if escaped {
                escaped = false
            } else if c == '\\' {
                escaped = true
            } else if c == '"' {
                quote = false
            }
        } else if c == '"' {
            quote = true
        } else if line[i..].starts_with("//") {
            return &line[..i];
        }
    }
    line
}

pub fn arrow(text: &str) -> Option<usize> {
    let mut depth = 0;
    let mut quote = false;
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        if quote {
            if escaped {
                escaped = false
            } else if c == '\\' {
                escaped = true
            } else if c == '"' {
                quote = false
            };
            continue;
        }
        if c == '"' {
            quote = true;
            continue;
        }
        if matches!(c, '(' | '[' | '{' | '<') {
            depth += 1
        } else if matches!(c, ')' | ']' | '}' | '>') {
            depth -= 1
        }
        if depth == 0 && text[i..].starts_with(" -> ") {
            return Some(i);
        }
    }
    None
}
