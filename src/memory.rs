//! Object-relative stack, heap and global memory. Every byte has a value, an
//! initialization bit and a poison bit: an unwritten (undef) byte is
//! undefined bit by bit, while a byte of a stored poison value poisons any
//! integer loaded over it. Loads/stores check object bounds, alignment and
//! lifetime. Stored pointers keep provenance in guarded offset shadow slots.

use crate::value::{Kind, Value, and, bv};
use std::collections::{BTreeMap, BTreeSet};

/// Initialization array of an object whose every byte is defined, and the
/// poison array of an object that never held poison. Kept literal where
/// possible so that loads read `true`/`false` instead of an array lookup and
/// most definedness obligations need no solver query.
pub const ALL_DEFINED: &str = "((as const (Array (_ BitVec 64) Bool)) true)";
pub const NONE_DEFINED: &str = "((as const (Array (_ BitVec 64) Bool)) false)";

/// A whole pointer stored at an object-relative offset. `valid` guards its
/// presence after symbolic copies and overlapping writes; address equality
/// alone cannot create provenance. Loads additionally re-prove the bytes.
#[derive(Clone, Debug)]
pub struct PointerSlot {
    pub offset: String,
    pub value: Value,
    pub valid: String,
}

mod length;
pub use length::Length;

#[derive(Clone, Debug)]
pub struct Object {
    pub size: Length,
    pub base: String,
    pub readonly: bool,
    /// Entry objects remain disjoint even from other readonly objects.
    pub unique: bool,
    pub writeonly: bool,
    pub alignment: u64,
    pub bytes: String,
    pub initialized: String,
    /// Bytes holding (part of) a stored poison value.
    pub poison: String,
    /// Fabricated integer pointers reserve no address range.
    pub allocated: bool,
    pub alive: bool,
    pub stack: bool,
    pub frame_live: bool,
    pub managed: bool,
    /// Allocated through the allocator ABI; only these may be freed.
    pub heap: bool,
    /// Provenance at concrete or symbolic offsets, with presence guards.
    pub pointers: Vec<PointerSlot>,
    /// Set for a function's code object: (module, symbol) it denotes.
    pub function: Option<(String, String)>,
    /// Integer values stored whole at concrete offsets whose bytes no later
    /// write touched: a load of exactly that range forwards the value itself
    /// instead of re-reading the arrays, so repeated loads are one term.
    pub values: BTreeMap<u64, Value>,
    /// Offsets holding bytes derived from the OS random source, and whether
    /// such bytes may sit anywhere (a symbolic-offset write); see opaque.rs.
    pub tainted: BTreeSet<u64>,
    pub tainted_anywhere: bool,
    /// Offsets written as defined by concrete-offset stores since the
    /// object's contents were last all undefined; `None` stops tracking.
    /// When it covers the object, `initialized` becomes `ALL_DEFINED`.
    pub known: Option<BTreeSet<u64>>,
}
pub type Memory = BTreeMap<String, Object>;

impl Object {
    /// Byte `address`'s initialization condition.
    pub fn init_at(&self, address: &str) -> String {
        if self.initialized == ALL_DEFINED {
            "true".into()
        } else {
            format!("(select {} {address})", self.initialized)
        }
    }
    /// Byte `address`'s poison condition.
    pub fn poison_at(&self, address: &str) -> String {
        if self.poison == NONE_DEFINED {
            "false".into()
        } else {
            format!("(select {} {address})", self.poison)
        }
    }
    /// Records byte `address` (literally `offset` when concrete) as holding
    /// initialization `init` and poison `poison`. Every writer of the two
    /// arrays goes through here so the literal shortcuts stay sound.
    pub fn set_flags(&mut self, address: &str, offset: Option<u64>, init: &str, poison: &str) {
        if !(init == "true" && self.initialized == ALL_DEFINED) {
            self.initialized = format!("(store {} {address} {init})", self.initialized);
        }
        if !(poison == "false" && self.poison == NONE_DEFINED) {
            self.poison = format!("(store {} {address} {poison})", self.poison);
        }
        match (&mut self.known, offset) {
            (Some(known), Some(offset)) if init == "true" => {
                known.insert(offset);
            }
            (Some(known), Some(offset)) => {
                known.remove(&offset);
            }
            // A symbolic write can only add definedness when it is true.
            (Some(_), None) if init != "true" => self.known = None,
            _ => {}
        }
        if self.size.upper() > 0
            && let Some(known) = &self.known
            && known.len() as u64 >= self.size.upper()
        {
            self.initialized = ALL_DEFINED.into();
            self.known = None;
        }
    }
    /// All bytes undefined (a new lifetime or allocation).
    pub fn reset_flags(&mut self) {
        self.initialized = NONE_DEFINED.into();
        self.poison = NONE_DEFINED.into();
        self.known = Some(BTreeSet::new());
        self.values.clear();
        self.pointers.clear();
    }
    /// Drops forwarded values overlapping `count` bytes at `start`, or all
    /// of them for a write at an unknown offset.
    pub fn forget(&mut self, start: Option<u64>, count: u64) {
        match start {
            Some(start) => self.values.retain(|offset, value| {
                let width = value.width().map_or(u64::MAX, |w| u64::from(w / 8));
                offset.saturating_add(width) <= start || *offset >= start.saturating_add(count)
            }),
            None => self.values.clear(),
        }
    }

    /// A pointer/byte-copy/fill write replaces the provenance in its range.
    /// For symbolic ranges retain old slots only under a disjointness guard.
    /// The writer has already checked bounds, so live ranges do not wrap.
    pub fn forget_pointers(&mut self, start: &str, count: &str) {
        if crate::heap::literal(count) == Some(0) {
            return;
        }
        for slot in &mut self.pointers {
            let disjoint = match (
                crate::heap::literal(&slot.offset),
                crate::heap::literal(start),
                crate::heap::literal(count),
            ) {
                (Some(p), Some(s), Some(n)) => (p + 8 <= s || p >= s + n).to_string(),
                _ => format!(
                    "(or (= {count} {}) (bvule (bvadd {} {}) {start}) (bvule (bvadd {start} {count}) {}))",
                    bv(0, 64),
                    slot.offset,
                    bv(8, 64),
                    slot.offset
                ),
            };
            slot.valid = and(&[slot.valid.clone(), disjoint]);
        }
        self.pointers.retain(|slot| slot.valid != "false");
    }
}

/// A load may read an allocated dead stack object: its result is poison.
pub fn access_load(
    memory: &Memory,
    pointer: &Value,
    size: u64,
    alignment: u64,
) -> Result<String, String> {
    access_kind(memory, pointer, size, alignment, true, false)
}

pub fn access(
    memory: &Memory,
    pointer: &Value,
    size: u64,
    alignment: u64,
) -> Result<String, String> {
    access_kind(memory, pointer, size, alignment, false, false)
}

pub fn access_store(
    memory: &Memory,
    pointer: &Value,
    size: u64,
    alignment: u64,
) -> Result<String, String> {
    access_kind(memory, pointer, size, alignment, false, true)
}

fn access_kind(
    memory: &Memory,
    pointer: &Value,
    size: u64,
    alignment: u64,
    load: bool,
    write: bool,
) -> Result<String, String> {
    let (object, offset) = pointer.pointer()?;
    let object = memory
        .get(object)
        .ok_or_else(|| format!("unknown allocation {object}"))?;
    let mut valid = vec![
        pointer.defined.clone(),
        (object.alive || load && object.stack && object.frame_live).to_string(),
        (!load || !object.writeonly).to_string(),
        (!write || !object.readonly).to_string(),
    ];
    if size > object.size.upper() {
        valid.push("false".into());
    } else {
        let length = object.size.term();
        valid.push(format!("(bvuge {length} {})", bv(u128::from(size), 64)));
        valid.push(format!(
            "(bvule {offset} (bvsub {length} {}))",
            bv(u128::from(size), 64)
        ));
    }
    if alignment > 1 {
        // The ADDRESS must be aligned (LangRef: overstating an access's
        // alignment is undefined behavior). The base is a multiple of the
        // object's own alignment, so when that covers the access the offset
        // decides alone; otherwise the placement decides, as for the aligned
        // words core::slice::memchr reads from a byte string after
        // `align_offset`.
        let address = if object.alignment >= alignment {
            offset.to_owned()
        } else {
            format!("(bvadd {} {offset})", object.base)
        };
        valid.push(format!(
            "(= (bvurem {address} {}) {})",
            bv(u128::from(alignment), 64),
            bv(0, 64)
        ));
    }
    Ok(and(&valid))
}

pub fn load(memory: &Memory, pointer: &Value, width: u32) -> Result<Value, String> {
    if width == 0 || !width.is_multiple_of(8) {
        return Err("non-byte-sized memory load is unsupported".into());
    }
    let (name, offset) = pointer.pointer()?;
    let object = memory.get(name).ok_or("unknown allocation")?;
    if object.stack && !object.alive {
        return Ok(Value::bits(bv(0, width), width, "false".into()));
    }
    // A whole stored value read back unchanged: forward it, with exactly the
    // definedness the arrays would give (its bytes, plus the pointer's).
    if let Some(start) = crate::heap::literal(offset).and_then(|n| u64::try_from(n).ok())
        && let Some(stored) = object.values.get(&start)
        && stored.width().ok() == Some(width)
    {
        let (flags, poison) = crate::partial::store_flags(stored, (width / 8) as usize)?;
        let poison = crate::value::or(&[poison, crate::value::not(&pointer.defined)]);
        return Ok(crate::partial::bytes(stored.expr.clone(), flags, poison));
    }
    let mut values = Vec::new();
    let mut flags = Vec::new();
    let mut poison = vec![crate::value::not(&pointer.defined)];
    for index in 0..width / 8 {
        let address = format!("(bvadd {offset} {})", bv(u128::from(index), 64));
        values.push(format!("(select {} {address})", object.bytes));
        flags.push(object.init_at(&address));
        poison.push(object.poison_at(&address));
    }
    let mut expr = values.pop().ok_or("empty load")?;
    while let Some(byte) = values.pop() {
        expr = format!("(concat {expr} {byte})");
    }
    Ok(crate::partial::bytes(
        expr,
        flags,
        crate::value::or(&poison),
    ))
}

/// Stores `value`: per-byte definedness from a `Kind::Bytes` copy, else the
/// whole value's; an undefined non-copy value is poison in every byte.
pub fn store(memory: &mut Memory, pointer: &Value, value: &Value) -> Result<(), String> {
    let width = value.width()?;
    if width == 0 || !width.is_multiple_of(8) {
        return Err("non-byte-sized memory store is unsupported".into());
    }
    let (flags, poison) = crate::partial::store_flags(value, (width / 8) as usize)?;
    let (name, offset) = pointer.pointer()?;
    let start = crate::heap::literal(offset).and_then(|n| u64::try_from(n).ok());
    let object = memory.get_mut(name).ok_or("unknown allocation")?;
    if object.readonly {
        return Err("write to constant global memory is unsupported".into());
    }
    for (index, flag) in flags.iter().enumerate() {
        let address = format!("(bvadd {offset} {})", bv(index as u128, 64));
        let low = index as u32 * 8;
        let high = low + 7;
        object.bytes = format!(
            "(store {} {address} ((_ extract {high} {low}) {}))",
            object.bytes, value.expr
        );
        object.set_flags(&address, start.map(|s| s + index as u64), flag, &poison);
    }
    object.forget(start, u64::from(width / 8));
    if let Some(start) = start
        && matches!(value.kind, Kind::Bits(_) | Kind::Bytes { .. })
    {
        object.values.insert(start, value.clone());
    }
    Ok(())
}

pub fn gep(
    memory: &Memory,
    pointer: &Value,
    index: &Value,
    inbounds: bool,
    nuw: bool,
) -> Result<Value, String> {
    if index.width()? != 64 {
        return Err("GEP requires a 64-bit byte index".into());
    }
    let (object, offset) = pointer.pointer()?;
    let result = format!("(bvadd {offset} {})", index.expr);
    let allocation = memory.get(object).ok_or("unknown allocation")?;
    let mut valid = vec![pointer.defined.clone(), index.defined.clone()];
    if inbounds {
        let size = allocation.size.term();
        valid.push(format!("(bvule {offset} {size})"));
        valid.push(format!("(bvule {result} {size})"));
    }
    if nuw {
        valid.push(format!("(bvuge {result} {offset})"));
        // With an inbounds result, nonwrapping allocation base and unsigned
        // offset addition, absolute address no-wrap follows. Keep the full
        // obligation for GEP without inbounds. Regression proves implication.
        if !inbounds {
            valid.push(format!(
                "(bvuge (bvadd {} {}) {})",
                pointer.expr, index.expr, pointer.expr
            ));
        }
    }
    Ok(Value {
        expr: format!("(bvadd {} {result})", allocation.base),
        kind: Kind::Pointer {
            object: object.to_owned(),
            offset: result,
            bounded: inbounds,
        },
        defined: and(&valid),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(size: u64) -> Object {
        Object {
            allocated: true,
            size: size.into(),
            base: "b".into(),
            readonly: false,
            unique: false,
            writeonly: false,
            alignment: 1,
            bytes: "bytes".into(),
            initialized: NONE_DEFINED.into(),
            poison: NONE_DEFINED.into(),
            alive: true,
            stack: false,
            frame_live: true,
            managed: false,
            heap: false,
            pointers: Vec::new(),
            function: None,
            known: Some(BTreeSet::new()),
            values: BTreeMap::new(),
            tainted: BTreeSet::new(),
            tainted_anywhere: false,
        }
    }
    #[test]
    fn flags_stay_literal_until_they_cannot() {
        let mut o = object(2);
        o.set_flags("a0", Some(0), "true", "false");
        assert_eq!(o.poison, NONE_DEFINED);
        assert_ne!(o.initialized, ALL_DEFINED);
        // Covering every byte collapses initialization to the literal.
        o.set_flags("a1", Some(1), "true", "false");
        assert_eq!(o.initialized, ALL_DEFINED);
        assert_eq!(o.init_at("x"), "true");
        assert_eq!(o.poison_at("x"), "false");
        // An undefined write is recorded on top of the literal.
        o.set_flags("a0", Some(0), "false", "p");
        assert_eq!(
            o.init_at("x"),
            format!("(select (store {ALL_DEFINED} a0 false) x)")
        );
        assert_ne!(o.poison, NONE_DEFINED);
    }
    #[test]
    fn uncertain_writes_stop_tracking() {
        let mut o = object(2);
        o.set_flags("a0", Some(0), "true", "false");
        // A symbolic-offset write that may undefine a byte ends tracking...
        o.set_flags("s", None, "d", "false");
        assert!(o.known.is_none());
        // ...so later full coverage no longer collapses.
        o.set_flags("a0", Some(0), "true", "false");
        o.set_flags("a1", Some(1), "true", "false");
        assert_ne!(o.initialized, ALL_DEFINED);
        // An overwrite with an undefined value removes the byte.
        let mut p = object(2);
        p.set_flags("a0", Some(0), "true", "false");
        p.set_flags("a0", Some(0), "u", "false");
        p.set_flags("a1", Some(1), "true", "false");
        assert_ne!(p.initialized, ALL_DEFINED);
    }
    fn at(offset: &str, defined: &str) -> Value {
        Value {
            expr: format!("(bvadd base {offset})"),
            kind: Kind::Pointer {
                object: "o".into(),
                offset: offset.into(),
                bounded: true,
            },
            defined: defined.into(),
        }
    }
    #[test]
    fn forwarding_needs_an_untouched_exact_range() {
        let mut memory = Memory::new();
        memory.insert("o".into(), object(8));
        let zero = bv(0, 64);
        store(
            &mut memory,
            &at(&zero, "true"),
            &Value::bits("v".into(), 32, "d".into()),
        )
        .unwrap();
        // Same offset and width: the stored term, its definedness joined
        // with the pointer's.
        let loaded = load(&memory, &at(&zero, "q"), 32).unwrap();
        assert_eq!(loaded.expr, "v");
        assert!(loaded.defined.contains('d') && loaded.defined.contains('q'));
        // A narrower read goes through the arrays.
        assert_ne!(load(&memory, &at(&zero, "true"), 16).unwrap().expr, "v");
        // An overlapping write ends forwarding; a disjoint one does not.
        let mut touched = memory.clone();
        store(
            &mut touched,
            &at(&bv(3, 64), "true"),
            &Value::bits("w".into(), 8, "true".into()),
        )
        .unwrap();
        assert_ne!(load(&touched, &at(&zero, "true"), 32).unwrap().expr, "v");
        store(
            &mut memory,
            &at(&bv(4, 64), "true"),
            &Value::bits("w".into(), 8, "true".into()),
        )
        .unwrap();
        assert_eq!(load(&memory, &at(&zero, "true"), 32).unwrap().expr, "v");
        // A write at an unknown offset may overlap anything.
        store(
            &mut memory,
            &at("s", "true"),
            &Value::bits("w".into(), 8, "true".into()),
        )
        .unwrap();
        assert_ne!(load(&memory, &at(&zero, "true"), 32).unwrap().expr, "v");
    }
}
