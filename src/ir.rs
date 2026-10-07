//! LLVM function/block boundaries and comma-aware operands. Metadata and
//! attributes are retained for the evaluator; arbitrary unsupported syntax
//! is not guessed into an instruction.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub number: usize,
    /// `!noundef` metadata: loading undefined bytes is then undefined
    /// behavior; without it the loaded value is just poison.
    pub noundef: bool,
}
#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    /// Module that defines the function: "user" or a standard library crate.
    pub module: String,
    /// Integer parameters, used as symbolic inputs of an entry function.
    pub params: Vec<(String, u32)>,
    /// Every parameter as (name, type), used to bind call arguments.
    pub arguments: Vec<(String, String)>,
    /// Attribute words of each parameter (`noundef`, `dereferenceable(8)`).
    pub parameter_attributes: Vec<String>,
    pub blocks: BTreeMap<String, Vec<Line>>,
    pub entry: String,
    pub locations: BTreeMap<usize, crate::debug::Location>,
    /// Functions carrying `noreturn`, inline or via an attribute group.
    pub noreturn: BTreeSet<String>,
    /// Header text before `@name`: return attributes such as `range(...)`,
    /// `nonnull` and `align N` that make a violating result poison.
    pub result_attributes: String,
}

#[derive(Clone, Debug)]
pub struct Global {
    pub name: String,
    pub bytes: Vec<u8>,
    /// Pointers inside the initializer, resolved when the global is allocated.
    pub relocations: Vec<crate::constants::Relocation>,
    /// Struct padding and `undef` offsets, left uninitialized.
    pub padding: Vec<u64>,
    pub alignment: u64,
    /// `global` rather than `constant`: writable, starting at its initializer.
    pub mutable: bool,
    pub thread_local: bool,
    /// `private`/`internal`: never the target of another module's reference.
    pub local: bool,
}

impl Global {
    /// The symbol without `@` and quotes, as the library index spells it.
    pub fn symbol(&self) -> &str {
        self.name.trim_start_matches('@').trim_matches('"')
    }
}

/// One LLVM module indexed once; function bodies are parsed on demand.
pub struct Module {
    pub name: String,
    text: String,
    /// Byte offset, zero-based line and module-local linkage of every `define`.
    defines: BTreeMap<String, (usize, usize, bool)>,
    pub noreturn: BTreeSet<String>,
    /// Defined globals by bare symbol.
    pub globals: BTreeMap<String, Global>,
    /// Every global this module defines, including ones whose initializer
    /// is unsupported: such a name must never resolve to another module.
    pub global_definitions: BTreeSet<String>,
    /// Functions declared (not defined) in this module.
    pub declared: BTreeSet<String>,
    /// Weak declarations may be null when the linker does not provide them.
    pub weak: BTreeSet<String>,
    /// Named type bodies (`%T = type { ... }`) by name including `%`.
    pub types: crate::constants::Types,
    debug: Option<crate::debug::DebugInfo>,
}

impl Module {
    /// `debug` parses source locations; library modules skip that cost.
    pub fn new(name: &str, text: String, debug: bool) -> Result<Self, String> {
        let mut defines = BTreeMap::new();
        let mut declared = BTreeSet::new();
        let mut weak = BTreeSet::new();
        let mut offset = 0;
        for (number, line) in text.split_inclusive('\n').enumerate() {
            // Top-level entities are indexed by their first column; an
            // indented one would be silently missed, so it is refused.
            let body = line.trim_start();
            if body.len() != line.len()
                && (body.starts_with("define ")
                    || body.starts_with("declare ")
                    || body.starts_with('@'))
            {
                return Err(format!("indented top-level entity on line {}", number + 1));
            }
            if line.starts_with("define ")
                && let Some(symbol) = symbol_name(line)
            {
                defines.insert(symbol.to_owned(), (offset, number, local_linkage(line)));
            } else if line.starts_with("declare ")
                && let Some(symbol) = symbol_name(line)
            {
                declared.insert(symbol.to_owned());
                if line
                    .split('@')
                    .next()
                    .is_some_and(|h| h.split_whitespace().any(|w| w == "extern_weak"))
                {
                    weak.insert(symbol.to_owned());
                }
            }
            offset += line.len();
        }
        let types: crate::constants::Types = text
            .lines()
            .filter(|l| l.starts_with('%'))
            .filter_map(|l| l.split_once(" = type "))
            .map(|(name, body)| (name.to_owned(), body.trim().to_owned()))
            .collect();
        Ok(Self {
            name: name.to_owned(),
            noreturn: noreturn(&text),
            globals: globals(&text, &types),
            global_definitions: text
                .lines()
                .filter(|l| l.starts_with('@'))
                .filter_map(global_definition)
                .map(str::to_owned)
                .collect(),
            types,
            declared,
            weak,
            debug: debug.then(|| crate::debug::DebugInfo::new(&text)),
            defines,
            text,
        })
    }
    pub fn defines(&self, name: &str) -> bool {
        self.defines.contains_key(name)
    }
    /// True when `name` is a function defined or declared in this module.
    pub fn names_function(&self, name: &str) -> bool {
        self.defines.contains_key(name) || self.declared.contains(name)
    }
    /// Defined with external visibility to other modules (not private/internal).
    pub fn exports(&self, name: &str) -> bool {
        self.defines.get(name).is_some_and(|d| !d.2)
    }
    pub fn function(&self, requested: &str) -> Result<Function, String> {
        let &(offset, first, _) = self.defines.get(requested).ok_or_else(|| {
            format!(
                "LLVM entry {requested} not found; use an exported integer-argument bool function"
            )
        })?;
        let mut lines = self.text[offset..].lines();
        let header = lines.next().ok_or("function header missing")?;
        // Return attributes such as `range(i16 0, 5)` precede the name, so
        // the parameter list is the parenthesis right after `@name`.
        let at = header.find('@').ok_or("function name missing")?;
        let name_end = match header[at + 1..].strip_prefix('"') {
            Some(quoted) => at + 2 + quoted.find('"').ok_or("function name unterminated")? + 1,
            None => {
                at + header[at..]
                    .find('(')
                    .ok_or("function parameters missing")?
            }
        };
        let open = name_end
            + header[name_end..]
                .find('(')
                .ok_or("function parameters missing")?;
        let close =
            crate::symbols::matching_paren(header, open).ok_or("function parameters missing")?;
        let mut params = Vec::new();
        let mut arguments = Vec::new();
        let mut parameter_attributes = Vec::new();
        for param in split(&header[open + 1..close]) {
            if param.is_empty() {
                continue;
            }
            let words: Vec<_> = param.split_whitespace().collect();
            let ty = *words.first().ok_or("argument type missing")?;
            let name = words
                .iter()
                .find(|w| w.starts_with('%'))
                .ok_or("argument name missing")?
                .to_string();
            if let Some(width) = ty.strip_prefix('i').and_then(|s| s.parse().ok()) {
                params.push((name.clone(), width));
            }
            parameter_attributes.push(words[1..words.len().saturating_sub(1)].join(" "));
            arguments.push((name, ty.to_owned()));
        }
        // An unlabeled entry block takes the next unnamed value number.
        let numbered = arguments
            .iter()
            .filter(|(n, _)| n[1..].bytes().all(|b| b.is_ascii_digit()))
            .count();
        let mut blocks = BTreeMap::new();
        let mut locations = BTreeMap::new();
        let mut block = String::new();
        let mut entry = String::new();
        // A switch spans lines until its closing bracket; keep it as one line.
        let mut open_switch: Option<(String, usize)> = None;
        for (i, source) in lines.enumerate().map(|(i, l)| (first + 1 + i, l)) {
            if source == "}" {
                break;
            }
            let mut code = strip_comment(source).trim().to_owned();
            let mut number = i;
            if let Some((text, start)) = open_switch.take() {
                let joined = format!("{text} {code}");
                if !code.starts_with(']') && !code.starts_with("to label ") {
                    open_switch = Some((joined, start));
                    continue;
                }
                code = joined;
                number = start;
            } else if code.contains("switch ") && code.ends_with('[') {
                open_switch = Some((code, i));
                continue;
            } else if is_invoke(&code) && !code.contains(" to label ") {
                // `invoke` prints its `to label ... unwind label ...` on the
                // next line; joined like a switch.
                open_switch = Some((code, i));
                continue;
            }
            let code = code.as_str();
            if code.is_empty() {
                continue;
            }
            if let Some(label) = code.strip_suffix(':') {
                block = label.to_owned();
                if entry.is_empty() {
                    entry = block.clone();
                }
                blocks.entry(block.clone()).or_insert_with(Vec::new);
            } else {
                if block.is_empty() {
                    block = numbered.to_string();
                    entry = block.clone();
                    blocks.insert(block.clone(), Vec::new());
                }
                if let Some(location) = self.debug.as_ref().and_then(|d| d.location(source)) {
                    locations.insert(number + 1, location);
                }
                let (code, noundef) = split_metadata(code);
                let lines = blocks
                    .get_mut(&block)
                    .ok_or("instruction outside a block")?;
                // A panic is a reported failure, so only the normal edge of an
                // `invoke` (std is built with unwinding) can continue: it is
                // a call followed by a branch to the normal label.
                if let Some((call, normal)) = split_invoke(code) {
                    lines.push(Line {
                        text: call,
                        number: number + 1,
                        noundef,
                    });
                    lines.push(Line {
                        text: format!("br label {normal}"),
                        number: number + 1,
                        noundef: false,
                    });
                    continue;
                }
                lines.push(Line {
                    text: code.to_owned(),
                    number: number + 1,
                    noundef,
                });
            }
        }
        Ok(Function {
            name: requested.to_owned(),
            module: self.name.clone(),
            params,
            arguments,
            parameter_attributes,
            blocks,
            entry,
            locations,
            noreturn: self.noreturn.clone(),
            result_attributes: header[..header.find('@').unwrap_or(0)].to_owned(),
        })
    }
}

fn is_invoke(code: &str) -> bool {
    code.starts_with("invoke ") || code.contains(" = invoke ")
}

/// `[%x = ]invoke CALL to label %normal unwind label %u` as the call text
/// `[%x = ]call CALL` and the normal label.
fn split_invoke(code: &str) -> Option<(String, String)> {
    if !is_invoke(code) {
        return None;
    }
    let (call, edges) = code.rsplit_once(" to label ")?;
    let (normal, _) = edges.split_once(" unwind ")?;
    let call = match call.split_once(" = invoke ") {
        Some((name, rest)) => format!("{name} = call {rest}"),
        None => format!("call {}", call.strip_prefix("invoke ")?),
    };
    Some((call, normal.trim().to_owned()))
}

/// The first word after `define ` or after `@name = `: the linkage when
/// one is printed (default external linkage is not).
fn linkage(line: &str) -> Option<&str> {
    let rest = match line.strip_prefix("define ") {
        Some(rest) => rest,
        None => line.split_once(" = ")?.1,
    };
    rest.split_whitespace().next()
}

/// `private` and `internal` definitions are visible only inside their module.
pub fn local_linkage(line: &str) -> bool {
    linkage(line).is_some_and(|w| matches!(w, "private" | "internal"))
}

/// `available_externally`: a copy of a definition that lives elsewhere.
pub fn available_externally(line: &str) -> bool {
    linkage(line) == Some("available_externally")
}

/// Linkage of a definition whose copies are interchangeable across modules.
pub fn odr_linkage(line: &str) -> bool {
    linkage(line).is_some_and(|w| matches!(w, "linkonce_odr" | "weak_odr" | "available_externally"))
}

/// Bare symbol of a global definition line (`@name = ...`); `None` for a
/// declaration, which has no contents of its own.
pub fn global_definition(line: &str) -> Option<&str> {
    let (name, _) = line.split_once(" = ")?;
    if !name.starts_with('@') || matches!(linkage(line), Some("external" | "extern_weak")) {
        return None;
    }
    Some(name.trim_start_matches('@').trim_matches('"'))
}

/// An instruction without its metadata attachments (`, !dbg !7`), and
/// whether they include `!noundef`. Attachments start at the first
/// top-level comma followed by `!`, however it is spaced.
fn split_metadata(code: &str) -> (&str, bool) {
    let (mut depth, mut quoted) = (0i32, false);
    for (i, byte) in code.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            _ if quoted => {}
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth -= 1,
            b',' if depth == 0 && code[i + 1..].trim_start().starts_with('!') => {
                let noundef = code[i + 1..].split(',').any(|attachment| {
                    attachment
                        .trim_start()
                        .strip_prefix("!noundef")
                        .is_some_and(|rest| {
                            !rest.starts_with(|c: char| {
                                c.is_ascii_alphanumeric() || c == '_' || c == '.'
                            })
                        })
                });
                return (code[..i].trim_end(), noundef);
            }
            _ => {}
        }
    }
    (code, false)
}

/// The line before its `;` comment; a `;` inside quotes (a named type
/// such as `%"MaybeUninit<[u8; 8]>"`) is not a comment.
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, byte) in line.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b';' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Symbol of a `define`/`declare` line, without quotes.
fn symbol_name(line: &str) -> Option<&str> {
    let at = line.find('@')?;
    let open = at + line[at..].find('(')?;
    Some(line[at + 1..open].trim_matches('"'))
}

#[cfg(test)]
pub fn function(text: &str, requested: &str) -> Result<Function, String> {
    Module::new("user", text.to_owned(), true)?.function(requested)
}

fn noreturn(text: &str) -> BTreeSet<String> {
    let groups: BTreeSet<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("attributes #"))
        .filter(|l| l.split_whitespace().any(|w| w == "noreturn"))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    text.lines()
        .filter(|l| l.starts_with("declare ") || l.starts_with("define "))
        .filter_map(|l| {
            let at = l.find('@')?;
            let open = at + l[at..].find('(')?;
            let close = crate::symbols::matching_paren(l, open)?;
            let diverges = l[close + 1..].split_whitespace().any(|w| {
                w == "noreturn" || w.strip_prefix('#').is_some_and(|g| groups.contains(g))
            });
            diverges.then(|| l[at + 1..open].trim_matches('"').to_owned())
        })
        .collect()
}

/// Comma-separated operands at bracket depth zero. Quoted names such as
/// `%"Box<dyn Fn(u8) -> u16>"` are opaque: their brackets do not count.
pub fn split(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0isize;
    let mut start = 0;
    let mut quoted = false;
    for (i, byte) in text.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            _ if quoted => {}
            b'[' | b'(' | b'{' | b'<' => depth += 1,
            b']' | b')' | b'}' | b'>' => depth -= 1,
            b',' if depth == 0 => {
                result.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(text[start..].trim());
    result
}

pub fn typed(text: &str) -> Result<(&str, &str), String> {
    let text = text.trim();
    if text.starts_with(['{', '[', '<']) {
        // Struct, array and vector types nest; the type ends where the
        // brackets balance.
        let (mut depth, mut quoted) = (0i32, false);
        for (i, byte) in text.bytes().enumerate() {
            match byte {
                b'"' => quoted = !quoted,
                _ if quoted => {}
                b'{' | b'[' | b'<' => depth += 1,
                b'}' | b']' | b'>' => depth -= 1,
                _ => {}
            }
            if depth == 0 && !quoted {
                return Ok((&text[..=i], text[i + 1..].trim()));
            }
        }
        return Err("aggregate type missing bracket".into());
    }
    text.split_once(' ')
        .ok_or_else(|| format!("typed operand missing value: {text}"))
}

// Globals whose initializer this parser reads exactly. Others (unsupported
// layouts, other address spaces, definitions a link may replace) stay
// unavailable and are reported as unknown when actually referenced.
fn globals(text: &str, types: &crate::constants::Types) -> BTreeMap<String, Global> {
    text.lines()
        .filter(|l| l.starts_with('@'))
        .filter_map(|l| global(l, types))
        .map(|g| (g.symbol().to_owned(), g))
        .collect()
}

fn global(line: &str, types: &crate::constants::Types) -> Option<Global> {
    global_definition(line)?;
    let (name, rest) = line.split_once(" = ")?;
    let (mut mutable, mut thread_local, mut offset) = (None, false, 0);
    for word in rest.split(' ') {
        offset += word.len() + 1;
        match word {
            "global" => mutable = Some(true),
            "constant" => mutable = Some(false),
            _ if word.starts_with("thread_local") => thread_local = true,
            // The final link may pick another definition or merge these.
            "weak" | "linkonce" | "common" | "appending" | "extern_weak" => return None,
            _ if word.starts_with("addrspace") => return None,
            _ => {}
        }
        if mutable.is_some() {
            break;
        }
    }
    let mutable = mutable?;
    // A mutable copy of a definition that lives elsewhere is not that
    // object, and mutable ODR copies in several modules are one object after
    // linking, which per-module allocation would split.
    if mutable && odr_linkage(line) {
        return None;
    }
    let (constant, alignment) = crate::constants::global(rest.get(offset..)?, types)?;
    Some(Global {
        name: name.into(),
        bytes: constant.bytes,
        relocations: constant.relocations,
        padding: constant.padding,
        alignment,
        mutable,
        thread_local,
        local: local_linkage(line),
    })
}
