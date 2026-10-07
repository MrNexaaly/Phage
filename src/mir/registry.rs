//! Restricted declarations from rustc's expanded source: field order, enum
//! discriminants, generic names and Drop markers. Ambiguity is unsupported.

use super::parse::base_type;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Definition {
    pub fields: Vec<String>,
    pub variants: BTreeMap<String, i128>,
    pub supported: bool,
}
pub struct Registry {
    pub definitions: BTreeMap<String, Definition>,
    pub generics: BTreeMap<String, Vec<String>>,
    pub destructors: BTreeSet<String>,
    pub traits: BTreeSet<String>,
    pub custom_allocator: bool,
    pub constant_types: BTreeMap<(String, String), String>,
}

fn tokens(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1
            }
            result.push(text[start..i.min(bytes.len())].into());
            continue;
        }
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1
            }
            result.push(text[start..i].into());
        } else {
            result.push((bytes[i] as char).to_string());
            i += 1
        }
    }
    result
}
fn end(tokens: &[String], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(start) {
        if matches!(t.as_str(), "{" | "(" | "[" | "<") {
            depth += 1
        }
        if matches!(t.as_str(), "}" | ")" | "]" | ">") {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn parts(tokens: &[String]) -> Vec<&[String]> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, t) in tokens.iter().enumerate() {
        match t.as_str() {
            "{" | "(" | "[" | "<" => depth += 1,
            "}" | ")" | "]" | ">" => depth -= 1,
            "," if depth == 0 => {
                result.push(&tokens[start..i]);
                start = i + 1
            }
            _ => {}
        }
    }
    if start < tokens.len() {
        result.push(&tokens[start..])
    }
    result
}
fn without_attributes(mut part: &[String]) -> &[String] {
    while part.first().is_some_and(|s| s == "#") && part.get(1).is_some_and(|s| s == "[") {
        let Some(last) = end(part, 1) else { return &[] };
        part = &part[last + 1..];
    }
    part
}
fn integer(part: &[String]) -> Option<i128> {
    let text = part.join("").replace('_', "");
    if let Some(hex) = text.strip_prefix("0x") {
        i128::from_str_radix(hex, 16).ok()
    } else {
        text.parse().ok()
    }
}

impl Registry {
    pub fn new(text: &str) -> Result<Self, String> {
        let mut registry = Self {
            definitions: BTreeMap::new(),
            generics: BTreeMap::new(),
            destructors: BTreeSet::new(),
            traits: BTreeSet::new(),
            custom_allocator: false,
            constant_types: BTreeMap::new(),
        };
        let t = tokens(text);
        registry.custom_allocator = t.iter().any(|t| t == "global_allocator");
        let mut i = 0;
        let mut modules: Vec<(String, usize)> = Vec::new();
        while i < t.len() {
            while modules.last().is_some_and(|(_, close)| *close < i) {
                modules.pop();
            }
            let prefix = modules
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join("::");
            let key = |name: &str| {
                if prefix.is_empty() {
                    name.to_owned()
                } else {
                    format!("{prefix}::{name}")
                }
            };
            if t[i] == "mod"
                && t.get(i + 2).is_some_and(|s| s == "{")
                && let Some(close) = end(&t, i + 2)
            {
                modules.push((t[i + 1].clone(), close));
                i += 3;
                continue;
            }
            if matches!(t[i].as_str(), "struct" | "enum") {
                let Some(name) = t.get(i + 1) else { break };
                let enum_type = t[i] == "enum";
                let mut open = i + 2;
                if t.get(open).is_some_and(|s| s == "<") {
                    open = end(&t, open).ok_or("expanded generic declaration unclosed")? + 1;
                }
                if t.get(open).is_some_and(|s| matches!(s.as_str(), "{" | "(")) {
                    let close = end(&t, open).ok_or("expanded type declaration unclosed")?;
                    let mut definition = Definition {
                        fields: Vec::new(),
                        variants: BTreeMap::new(),
                        supported: true,
                    };
                    let mut next = 0i128;
                    for part in parts(&t[open + 1..close]) {
                        let part = without_attributes(part);
                        if part.is_empty() {
                            continue;
                        }
                        if enum_type {
                            let Some(variant) = part.first() else {
                                continue;
                            };
                            let tag = if let Some(eq) = part.iter().position(|p| p == "=") {
                                match integer(&part[eq + 1..]) {
                                    Some(n) => n,
                                    None => {
                                        definition.supported = false;
                                        0
                                    }
                                }
                            } else {
                                next
                            };
                            definition.variants.insert(variant.clone(), tag);
                            next = tag.checked_add(1).ok_or("enum discriminant overflow")?;
                        } else if t[open] == "{" {
                            if let Some(colon) = part.iter().position(|p| p == ":") {
                                if colon == 0 {
                                    definition.supported = false
                                } else {
                                    definition.fields.push(part[colon - 1].clone());
                                }
                            } else {
                                definition.supported = false
                            }
                        } else {
                            definition.fields.push(definition.fields.len().to_string());
                        }
                    }
                    let name = key(name);
                    if registry.definitions.insert(name, definition).is_some() {
                        return Err("ambiguous expanded type declaration".into());
                    }
                    i = close + 1;
                    continue;
                }
            }
            if t[i] == "trait"
                && let Some(name) = t.get(i + 1)
            {
                registry.traits.insert(name.clone());
            }
            if t[i] == "fn" && t.get(i + 2).is_some_and(|s| s == "<") {
                let close = end(&t, i + 2).ok_or("generic function parameters unclosed")?;
                for parameter in parts(&t[i + 3..close]) {
                    if parameter.first().is_some_and(|p| p == "const") && parameter.len() >= 4 {
                        registry.constant_types.insert(
                            (key(&t[i + 1]), parameter[1].clone()),
                            parameter[3..].join(""),
                        );
                    }
                }
                let names = parts(&t[i + 3..close])
                    .iter()
                    .filter_map(|p| {
                        if p.first().is_some_and(|s| s == "const") {
                            p.get(1).cloned()
                        } else if p.first().is_some_and(|s| s == "'") {
                            None
                        } else {
                            p.first().cloned()
                        }
                    })
                    .collect();
                registry.generics.insert(key(&t[i + 1]), names);
            }
            if t[i] == "impl" {
                let Some(open) = t[i..].iter().position(|s| s == "{").map(|p| p + i) else {
                    break;
                };
                let header = &t[i..open];
                if header.iter().any(|s| s == "Drop")
                    && let Some(for_pos) = header.iter().position(|s| s == "for")
                    && let Some(ty) = header.get(for_pos + 1)
                {
                    registry.destructors.insert(key(ty));
                }
            }
            i += 1;
        }
        Ok(registry)
    }
    pub fn definition(&self, ty: &str) -> Option<&Definition> {
        let base = base_type(ty);
        if let Some(def) = self.definitions.get(&base) {
            return Some(def);
        }
        if base.starts_with("std::") || base.starts_with("core::") {
            return None;
        }
        let leaf = base.rsplit("::").next()?;
        let matches: Vec<_> = self
            .definitions
            .iter()
            .filter(|(k, _)| k.rsplit("::").next() == Some(leaf))
            .map(|(_, v)| v)
            .collect();
        if matches.len() == 1 {
            Some(matches[0])
        } else {
            None
        }
    }
    pub fn variant(&self, ty: &str, name: &str) -> Option<i128> {
        if let Some(def) = self.definition(ty) {
            return def
                .supported
                .then(|| def.variants.get(name).copied())
                .flatten();
        }
        let base = base_type(ty);
        if !base.starts_with("std::")
            && !base.starts_with("core::")
            && self
                .definitions
                .keys()
                .any(|k| k.rsplit("::").next() == base.rsplit("::").next())
        {
            return None;
        }
        match (base.rsplit("::").next()?, name) {
            ("Option", "None") | ("Result", "Ok") => Some(0),
            ("Option", "Some") | ("Result", "Err") => Some(1),
            _ => None,
        }
    }
    pub fn has_drop(&self, ty: &str) -> bool {
        let base = base_type(ty);
        let leaf = base.rsplit("::").next();
        self.destructors
            .iter()
            .any(|t| t == &base || t.rsplit("::").next() == leaf)
    }
}
