//! Global initializers: byte strings, integers, null, zero initializers,
//! `undef` (uninitialized), packed and natural structs, arrays, and pointer relocations
//! (`ptr @symbol`, `ptr getelementptr inbounds (i8, ptr @symbol, i64 N)`).
//! Produces exact bytes plus relocations; anything else is rejected so the
//! global stays unavailable (unknown when referenced).

use std::collections::BTreeMap;

/// Named type bodies by name including `%`, as a module declares them.
pub type Types = BTreeMap<String, String>;

/// A pointer stored in a constant: byte offset, target symbol, byte addend.
#[derive(Clone, Debug, PartialEq)]
pub struct Relocation {
    pub offset: u64,
    pub symbol: String,
    pub addend: u64,
    /// From `getelementptr inbounds`: out of the target's bounds is poison.
    pub inbounds: bool,
}

/// Parses the part of a global definition after `constant `: the typed
/// initializer and an optional `, align N`.
pub fn global(rest: &str, types: &Types) -> Option<(Constant, u64)> {
    let rest = rest.trim();
    let (ty, tail) = split_type(rest)?;
    let tail = tail.trim_start();
    let end = if let Some(string) = tail.strip_prefix("c\"") {
        2 + string.find('"')? + 1
    } else if tail.starts_with(['[', '{', '<']) {
        matching_close(tail)? + 1
    } else {
        tail.find(',').unwrap_or(tail.len())
    };
    let (value, suffix) = tail.split_at(end);
    let constant = parse(&format!("{ty} {value}"), types)?;
    // Without `align` a definition still gets at least its type's ABI
    // alignment (LLVM's getPointerAlignment), and accesses are checked
    // against the address, so stating less would invent misalignment.
    let align = suffix
        .split(", ")
        .find_map(|part| part.trim().strip_prefix("align "))
        .and_then(|n| n.trim().parse().ok())
        .or_else(|| align_of(ty, types).map(u64::next_power_of_two))
        .unwrap_or(1);
    Some((constant, align))
}

/// Bytes, relocations and uninitialized offsets (struct padding and `undef`
/// parts, whose contents LLVM does not define) of an initializer.
pub struct Constant {
    pub bytes: Vec<u8>,
    pub relocations: Vec<Relocation>,
    pub padding: Vec<u64>,
}

/// Parses `TYPE VALUE`.
pub fn parse(text: &str, types: &Types) -> Option<Constant> {
    let (ty, value) = split_type(text.trim())?;
    let mut constant = Constant {
        bytes: Vec::new(),
        relocations: Vec::new(),
        padding: Vec::new(),
    };
    emit(ty, value.trim(), &mut constant, types)?;
    Some(constant)
}

fn emit(ty: &str, value: &str, out: &mut Constant, types: &Types) -> Option<()> {
    let ty = resolve(ty, types)?;
    let size = size_of(ty, types)?;
    if value == "undef" {
        // Contents the program never set: uninitialized, not zero.
        for _ in 0..size {
            out.padding.push(out.bytes.len() as u64);
            out.bytes.push(0);
        }
        return Some(());
    }
    if value == "zeroinitializer" || (value == "null" && ty == "ptr") {
        out.bytes
            .extend(std::iter::repeat_n(0, usize::try_from(size).ok()?));
        return Some(());
    }
    if ty == "ptr" {
        let (symbol, addend, inbounds) = pointer_target(value)?;
        out.relocations.push(Relocation {
            offset: out.bytes.len() as u64,
            symbol,
            addend,
            inbounds,
        });
        out.bytes.extend([0; 8]);
        return Some(());
    }
    if let Some(width) = ty.strip_prefix('i').and_then(|w| w.parse::<u32>().ok()) {
        let value: i128 = value.parse().ok()?;
        let raw = value.to_le_bytes();
        out.bytes.extend(&raw[..(width / 8) as usize]);
        return Some(());
    }
    if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let (_, element) = inner.split_once(" x ")?;
        if let Some(string) = value.strip_prefix("c\"").and_then(|v| v.strip_suffix('"')) {
            let decoded = decode(string)?;
            (decoded.len() as u64 == size).then_some(())?;
            out.bytes.extend(decoded);
            return Some(());
        }
        let items = value.strip_prefix('[')?.strip_suffix(']')?;
        for item in fields(items) {
            let (item_ty, item_value) = split_type(item)?;
            (item_ty == element.trim()).then_some(())?;
            emit(item_ty, item_value.trim(), out, types)?;
        }
        return Some(());
    }
    let (packed, inner_ty) = struct_fields(ty)?;
    let inner_value = if packed {
        value.strip_prefix("<{")?.strip_suffix("}>")?
    } else {
        value.strip_prefix('{')?.strip_suffix('}')?
    };
    let start = out.bytes.len() as u64;
    let field_types = fields(inner_ty);
    let values = fields(inner_value);
    (field_types.len() == values.len()).then_some(())?;
    for (field_ty, field) in field_types.iter().zip(values) {
        let (value_ty, field_value) = split_type(field)?;
        (value_ty == field_ty.trim()).then_some(())?;
        if !packed {
            let align = align_of(field_ty, types)?;
            while !(out.bytes.len() as u64 - start).is_multiple_of(align) {
                out.padding.push(out.bytes.len() as u64);
                out.bytes.push(0);
            }
        }
        emit(value_ty, field_value.trim(), out, types)?;
    }
    while (out.bytes.len() as u64 - start) < size {
        out.padding.push(out.bytes.len() as u64);
        out.bytes.push(0);
    }
    Some(())
}

/// A named type's body (`%T` to `{ ... }`), or `ty` itself when unnamed;
/// `None` for an unknown or opaque name.
pub fn resolve<'a>(ty: &'a str, types: &'a Types) -> Option<&'a str> {
    let mut ty = ty.trim();
    // Bodies may name other types; a by-value cycle cannot exist, the
    // bound only guards malformed input.
    for _ in 0..64 {
        if !ty.starts_with('%') {
            return Some(ty);
        }
        ty = types.get(ty).map(|b| b.trim()).filter(|b| *b != "opaque")?;
    }
    None
}

/// Allocation size under x86_64 data layout for the supported types.
pub fn size_of(ty: &str, types: &Types) -> Option<u64> {
    let ty = resolve(ty, types)?;
    if ty == "ptr" {
        return Some(8);
    }
    if let Some((e, s)) = crate::floats::format(ty) {
        return Some(u64::from(e + s) / 8);
    }
    if let Some(width) = ty.strip_prefix('i').and_then(|w| w.parse::<u64>().ok()) {
        // Values are encoded through i128; wider integers are unsupported.
        return (width % 8 == 0 && width.is_power_of_two() && width <= 128).then_some(width / 8);
    }
    if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let (count, element) = inner.split_once(" x ")?;
        return count
            .trim()
            .parse::<u64>()
            .ok()?
            .checked_mul(size_of(element, types)?);
    }
    let (packed, inner) = struct_fields(ty)?;
    let mut size = 0u64;
    let mut max_align = 1;
    for field in fields(inner) {
        let align = if packed { 1 } else { align_of(field, types)? };
        max_align = max_align.max(align);
        size = size.div_ceil(align) * align + size_of(field, types)?;
    }
    Some(size.div_ceil(max_align) * max_align)
}

fn align_of(ty: &str, types: &Types) -> Option<u64> {
    let ty = resolve(ty, types)?;
    if ty == "ptr" {
        return Some(8);
    }
    if let Some((e, s)) = crate::floats::format(ty) {
        return Some(u64::from(e + s) / 8);
    }
    if let Some(width) = ty.strip_prefix('i').and_then(|w| w.parse::<u64>().ok()) {
        return Some((width / 8).clamp(1, 16));
    }
    if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        return align_of(inner.split_once(" x ")?.1, types);
    }
    let (packed, inner) = struct_fields(ty)?;
    if packed {
        return Some(1);
    }
    fields(inner)
        .into_iter()
        .map(|field| align_of(field, types))
        .try_fold(1, |a, b| Some(a.max(b?)))
}

/// Byte offset and type of field `index` of a (possibly named) struct type,
/// using the same natural or packed layout as `size_of`.
pub fn field_offset<'a>(ty: &'a str, index: usize, types: &'a Types) -> Option<(u64, &'a str)> {
    let (packed, inner) = struct_fields(resolve(ty, types)?)?;
    let mut offset = 0u64;
    for (i, field) in fields(inner).into_iter().enumerate() {
        let align = if packed { 1 } else { align_of(field, types)? };
        offset = offset.div_ceil(align) * align;
        if i == index {
            return Some((offset, field));
        }
        offset += size_of(field, types)?;
    }
    None
}

/// Element type of a (possibly named) array type.
pub fn array_element<'a>(ty: &'a str, types: &'a Types) -> Option<&'a str> {
    let inner = resolve(ty, types)?.strip_prefix('[')?.strip_suffix(']')?;
    Some(inner.split_once(" x ")?.1.trim())
}

fn struct_fields(ty: &str) -> Option<(bool, &str)> {
    if let Some(inner) = ty.strip_prefix("<{").and_then(|t| t.strip_suffix("}>")) {
        return Some((true, inner));
    }
    ty.strip_prefix('{')
        .and_then(|t| t.strip_suffix('}'))
        .map(|inner| (false, inner))
}

/// A direct global pointer or the restricted constant GEP form accepted in
/// global initializers and instruction operands.
pub(crate) fn pointer_target(value: &str) -> Option<(String, u64, bool)> {
    if let Some(symbol) = value.strip_prefix('@') {
        return Some((format!("@{}", symbol.trim()), 0, false));
    }
    // getelementptr inbounds (i8, ptr @x, i64 N)
    let inner = value
        .strip_prefix("getelementptr inbounds (")
        .or_else(|| value.strip_prefix("getelementptr inbounds nuw ("))?
        .strip_suffix(')')?;
    let parts = fields(inner);
    match parts.as_slice() {
        [element, base, offset] if element.trim() == "i8" => {
            let symbol = base.trim().strip_prefix("ptr @")?;
            let addend = offset.trim().strip_prefix("i64 ")?.parse().ok()?;
            Some((format!("@{symbol}"), addend, true))
        }
        _ => None,
    }
}

/// Splits `TYPE VALUE` where the type may be bracketed.
fn split_type(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let end = if text.starts_with(['[', '{', '<']) {
        matching_close(text)? + 1
    } else if let Some(quoted) = text.strip_prefix("%\"") {
        2 + quoted.find('"')? + 1
    } else {
        text.find(' ')?
    };
    Some((&text[..end], &text[end..]))
}

fn matching_close(text: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut quoted = false;
    for (i, byte) in text.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'[' | b'{' | b'<' | b'(' if !quoted => depth += 1,
            b']' | b'}' | b'>' | b')' if !quoted => {
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

/// Comma-separated fields at nesting depth zero, ignoring quoted text.
fn fields(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let (mut depth, mut quoted, mut start) = (0i32, false, 0);
    for (i, byte) in text.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'[' | b'{' | b'<' | b'(' if !quoted => depth += 1,
            b']' | b'}' | b'>' | b')' if !quoted => depth -= 1,
            b',' if depth == 0 && !quoted => {
                result.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = text[start..].trim();
    if !last.is_empty() {
        result.push(last);
    }
    result
}

/// LLVM string escapes: `\\` is a backslash, `\XX` a hex byte.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chars = text.bytes();
    while let Some(byte) = chars.next() {
        if byte != b'\\' {
            bytes.push(byte);
            continue;
        }
        let hi = chars.next()?;
        if hi == b'\\' {
            bytes.push(b'\\');
            continue;
        }
        let lo = chars.next()?;
        let hex = |b: u8| (b as char).to_digit(16);
        bytes.push((hex(hi)? * 16 + hex(lo)?) as u8);
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Option<Constant> {
        super::parse(text, &Types::new())
    }

    #[test]
    fn named_types_resolve_through_the_module_table() {
        let types = Types::from([
            ("%inner".to_owned(), "{ i8, i32 }".to_owned()),
            (
                "%\"Cell<[u8; 2]>\"".to_owned(),
                "{ %inner, ptr }".to_owned(),
            ),
            ("%hidden".to_owned(), "opaque".to_owned()),
        ]);
        assert_eq!(size_of("%\"Cell<[u8; 2]>\"", &types), Some(16));
        assert_eq!(size_of("[3 x %inner]", &types), Some(24));
        assert_eq!(size_of("%hidden", &types), None);
        assert_eq!(size_of("%missing", &types), None);
        let constant = super::parse(
            "%\"Cell<[u8; 2]>\" { %inner { i8 7, i32 1 }, ptr null }",
            &types,
        )
        .unwrap();
        assert_eq!(constant.bytes[0], 7);
        assert_eq!(constant.padding, [1, 2, 3]);
    }

    #[test]
    fn vtable_bytes_and_relocations() {
        let constant = parse(
            "<{ [24 x i8], ptr }> <{ [24 x i8] c\"\\00\\00\\00\\00\\00\\00\\00\\00\\01\\00\\00\\00\\00\\00\\00\\00\\01\\00\\00\\00\\00\\00\\00\\00\", ptr @\"_ZN4area\" }>",
        )
        .unwrap();
        assert_eq!(constant.bytes.len(), 32);
        assert_eq!(constant.bytes[8], 1);
        assert!(constant.padding.is_empty());
        assert_eq!(
            constant.relocations,
            vec![Relocation {
                offset: 24,
                symbol: "@\"_ZN4area\"".into(),
                addend: 0,
                inbounds: false,
            }]
        );
    }
    #[test]
    fn struct_field_offsets_follow_natural_and_packed_layout() {
        let types = Types::default();
        let ty = "{ [8 x i16], i64 }";
        assert_eq!(field_offset(ty, 0, &types), Some((0, "[8 x i16]")));
        assert_eq!(field_offset(ty, 1, &types), Some((16, "i64")));
        assert_eq!(field_offset("{ i8, i32, i8 }", 1, &types), Some((4, "i32")));
        assert_eq!(field_offset("{ i8, i32, i8 }", 2, &types), Some((8, "i8")));
        assert_eq!(field_offset("<{ i8, i32 }>", 1, &types), Some((1, "i32")));
        assert_eq!(field_offset("{ i8 }", 1, &types), None);
        assert_eq!(array_element("[32 x { i64 }]", &types), Some("{ i64 }"));
    }

    #[test]
    fn natural_struct_padding_and_integers() {
        let constant = parse("{ i8, i32, ptr } { i8 1, i32 -2, ptr null }").unwrap();
        assert_eq!(
            constant.bytes,
            [1, 0, 0, 0, 254, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(constant.padding, [1, 2, 3]);
        assert!(constant.relocations.is_empty());
        let relocations = parse(
            "<{ ptr, [9 x i8] }> <{ ptr getelementptr inbounds (i8, ptr @s, i64 3), [9 x i8] c\"a,b\\22,cd\\\\x\" }>",
        )
        .unwrap()
        .relocations;
        assert_eq!(relocations[0].addend, 3);
        assert!(relocations[0].inbounds);
        assert!(parse("<{ ptr }> <{ ptr blockaddress(@f, %b) }>").is_none());
        // Integers wider than 128 bits are rejected, not a crash.
        assert!(parse("i256 0").is_none());
        // `undef` parts are uninitialized, not zero.
        let lazy = parse("<{ [2 x i8], [1 x i8] }> <{ [2 x i8] undef, [1 x i8] zeroinitializer }>")
            .unwrap();
        assert_eq!(lazy.bytes, [0, 0, 0]);
        assert_eq!(lazy.padding, [0, 1]);
    }
}
