//! Known-bits simplification of the terms the engine names. LLVM attaches
//! `nuw`, `nsw`, `disjoint` and similar flags exactly where its own
//! known-bits analysis proved them, and Phage turns each flag into a
//! definedness condition such as `(= (bvand (bvshl (zext b) 8) (zext a)) 0)`.
//! Such conditions are true, yet they ride along in every later query
//! (through hash values, per-byte memory flags, ...), and re-proving them
//! inside large queries made some hash-table obligations time out.
//!
//! This module computes known-zero/known-one bits of bit-vector terms with
//! LLVM's KnownBits transfer functions (carry analysis for addition), looking
//! through `define-fun` names, and rewrites terms bottom-up: fully known
//! bit-vectors become literals; comparisons decided by known bits or by the
//! bit ranges they imply become `true`/`false`; the shift-flag identities are
//! discharged; `and`/`or`/`not`/`ite` fold; and `select` reads over stores at
//! literal indices (or a chain storing one value everywhere) resolve. Every
//! rewrite is an equivalence; the tests check the transfer functions
//! exhaustively and the rewrites against Z3.

use std::{collections::BTreeMap, rc::Rc};

/// An S-expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Atom(String),
    List(Vec<Node>),
}

/// Terms longer than this are left alone (bounds recursion depth).
const TERM_LIMIT: usize = 8192;
/// Node visits per simplification; past it, nothing more is learned.
const WORK: usize = 20_000;

pub fn parse(text: &str) -> Option<Node> {
    if text.contains('|') || text.contains('"') {
        return None;
    }
    let mut stack: Vec<Vec<Node>> = vec![Vec::new()];
    let mut atom = String::new();
    let flush = |atom: &mut String, stack: &mut Vec<Vec<Node>>| {
        if !atom.is_empty()
            && let Some(top) = stack.last_mut()
        {
            top.push(Node::Atom(std::mem::take(atom)));
        }
    };
    for c in text.chars() {
        match c {
            '(' => {
                flush(&mut atom, &mut stack);
                stack.push(Vec::new());
            }
            ')' => {
                flush(&mut atom, &mut stack);
                let list = stack.pop()?;
                stack.last_mut()?.push(Node::List(list));
            }
            c if c.is_whitespace() => flush(&mut atom, &mut stack),
            c => atom.push(c),
        }
    }
    flush(&mut atom, &mut stack);
    let mut top = stack.pop()?;
    (stack.is_empty() && top.len() == 1)
        .then(|| top.pop())
        .flatten()
}

pub fn render(node: &Node) -> String {
    match node {
        Node::Atom(a) => a.clone(),
        Node::List(items) => {
            let parts: Vec<String> = items.iter().map(render).collect();
            format!("({})", parts.join(" "))
        }
    }
}

fn atom(text: &str) -> Node {
    Node::Atom(text.into())
}

fn head(node: &Node) -> Option<&str> {
    match node {
        Node::List(items) => match items.first() {
            Some(Node::Atom(a)) => Some(a),
            _ => None,
        },
        Node::Atom(_) => None,
    }
}

/// Bits known to be zero and known to be one in a `width`-bit value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Known {
    pub zero: u128,
    pub one: u128,
    pub width: u32,
    /// Unsigned bounds, at least as tight as the known bits imply.
    pub lo: u128,
    pub hi: u128,
}

pub fn mask(width: u32) -> u128 {
    if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}

impl Known {
    /// Known bits, with the bounds they imply.
    pub fn bits(zero: u128, one: u128, width: u32) -> Known {
        Known {
            zero,
            one,
            width,
            lo: one,
            hi: !zero & mask(width),
        }
    }
    /// Narrowed to `lo..=hi` as well (a contradiction keeps `self`).
    pub fn within(self, lo: u128, hi: u128) -> Known {
        let (lo, hi) = (self.lo.max(lo), self.hi.min(hi));
        if lo > hi {
            self
        } else {
            Known { lo, hi, ..self }
        }
    }
    pub fn exact(value: u128, width: u32) -> Known {
        let value = value & mask(width);
        Known::bits(!value & mask(width), value, width)
    }
    pub fn unknown(width: u32) -> Known {
        Known::bits(0, 0, width)
    }
    pub fn value(&self) -> Option<u128> {
        (self.lo == self.hi).then_some(self.lo)
    }
    pub fn min(&self) -> u128 {
        self.lo
    }
    pub fn max(&self) -> u128 {
        self.hi
    }
    fn not(self) -> Known {
        let m = mask(self.width);
        Known::bits(self.one, self.zero, self.width).within(m - self.hi, m - self.lo)
    }
    fn trailing_zeros(&self) -> u32 {
        (!self.zero).trailing_zeros().min(self.width)
    }
}

/// LLVM `KnownBits::computeForAddCarry`.
pub fn add_carry(a: Known, b: Known, carry_zero: bool, carry_one: bool) -> Known {
    let m = mask(a.width);
    // The bit-implied extremes (LLVM getMaxValue/getMinValue), not the bounds.
    let (a_max, b_max) = (!a.zero & m, !b.zero & m);
    let sum_zero = a_max
        .wrapping_add(b_max)
        .wrapping_add(u128::from(!carry_zero))
        & m;
    let sum_one = a
        .one
        .wrapping_add(b.one)
        .wrapping_add(u128::from(carry_one))
        & m;
    let carry_known_zero = !(sum_zero ^ a.zero ^ b.zero) & m;
    let carry_known_one = (sum_one ^ a.one ^ b.one) & m;
    let known = (a.zero | a.one) & (b.zero | b.one) & (carry_known_zero | carry_known_one);
    Known::bits(!sum_zero & known, sum_one & known, a.width)
}

pub fn shift(op: &str, x: Known, amount: u128) -> Known {
    let (w, m) = (x.width, mask(x.width));
    if amount >= u128::from(w) {
        return match op {
            "bvashr" => shift(op, x, u128::from(w - 1)),
            _ => Known::exact(0, w),
        };
    }
    let c = amount as u32;
    match op {
        "bvshl" => {
            let k = Known::bits(((x.zero << c) | mask(c)) & m, (x.one << c) & m, w);
            if x.hi <= m >> c {
                k.within(x.lo << c, x.hi << c)
            } else {
                k
            }
        }
        "bvlshr" => {
            Known::bits((x.zero >> c) | (!(m >> c) & m), x.one >> c, w).within(x.lo >> c, x.hi >> c)
        }
        _ => {
            let high = !(m >> c) & m;
            let sign = 1u128 << (w - 1);
            Known::bits(
                (x.zero >> c) | if x.zero & sign != 0 { high } else { 0 },
                (x.one >> c) | if x.one & sign != 0 { high } else { 0 },
                w,
            )
        }
    }
}

/// What the engine keeps between simplifications.
#[derive(Default)]
pub struct State {
    bodies: BTreeMap<String, String>,
    widths: BTreeMap<String, u32>,
    known: BTreeMap<String, Option<Known>>,
    parsed: BTreeMap<String, Option<Rc<Node>>>,
}

fn bitvector_width(sort: &str) -> Option<u32> {
    sort.strip_prefix("(_ BitVec ")?
        .trim_end_matches(')')
        .parse()
        .ok()
}

impl State {
    /// Bits of `name` fixed on every path that can mention it (the
    /// alignment of a fresh allocation's base, asserted where it is made).
    pub fn assume(&mut self, name: &str, known: Known) {
        self.known.insert(name.to_owned(), Some(known));
    }
    /// The `define-fun` body of `name`, if it is a defined name.
    pub fn body(&self, name: &str) -> Option<&str> {
        self.bodies.get(name).map(String::as_str)
    }
    /// A declared (free) symbol.
    pub fn declared(&mut self, name: &str, sort: &str) {
        if let Some(width) = bitvector_width(sort) {
            self.widths.insert(name.to_owned(), width);
        }
    }
    /// A `define-fun` name; its body is read lazily when a term mentions it.
    pub fn defined(&mut self, name: &str, sort: &str, body: &str) {
        self.declared(name, sort);
        self.bodies.insert(name.to_owned(), body.to_owned());
    }
    /// `expr` of `sort` rewritten; unchanged text when nothing applies.
    pub fn simplify(&mut self, expr: &str, sort: &str) -> String {
        if expr.len() > TERM_LIMIT || !expr.starts_with('(') {
            return expr.to_owned();
        }
        let Some(node) = parse(expr) else {
            return expr.to_owned();
        };
        let mut simplifier = Simplifier::new(
            &self.bodies,
            &self.widths,
            &mut self.known,
            &mut self.parsed,
        );
        let out = if sort == "Bool" {
            simplifier.boolean(&node)
        } else if bitvector_width(sort).is_some() {
            simplifier.bitvector(&node)
        } else {
            return expr.to_owned();
        };
        if out == node {
            expr.to_owned()
        } else {
            render(&out)
        }
    }
}

/// Simplification over the engine's `define-fun` bodies and symbol widths.
pub struct Simplifier<'a> {
    pub bodies: &'a BTreeMap<String, String>,
    pub widths: &'a BTreeMap<String, u32>,
    pub cache: &'a mut BTreeMap<String, Option<Known>>,
    parsed: &'a mut BTreeMap<String, Option<Rc<Node>>>,
    work: usize,
}

impl<'a> Simplifier<'a> {
    pub fn new(
        bodies: &'a BTreeMap<String, String>,
        widths: &'a BTreeMap<String, u32>,
        cache: &'a mut BTreeMap<String, Option<Known>>,
        parsed: &'a mut BTreeMap<String, Option<Rc<Node>>>,
    ) -> Self {
        Simplifier {
            bodies,
            widths,
            cache,
            parsed,
            work: WORK,
        }
    }

    fn tick(&mut self) -> bool {
        if self.work == 0 {
            return false;
        }
        self.work -= 1;
        true
    }

    /// The parsed body of a defined name (parsed once per engine).
    fn body(&mut self, name: &str) -> Option<Rc<Node>> {
        if let Some(node) = self.parsed.get(name) {
            return node.clone();
        }
        let node = self.bodies.get(name).and_then(|b| parse(b)).map(Rc::new);
        self.parsed.insert(name.to_owned(), node.clone());
        node
    }

    /// Known bits of a bit-vector term; `None` when even its width is unknown.
    pub fn known(&mut self, node: &Node) -> Option<Known> {
        if !self.tick() {
            return None;
        }
        match node {
            Node::Atom(a) => {
                if let Some(literal) = crate::heap::literal(a) {
                    let width = if let Some(hex) = a.strip_prefix("#x") {
                        4 * hex.len() as u32
                    } else {
                        a.len() as u32 - 2
                    };
                    return (width <= 128).then(|| Known::exact(literal, width));
                }
                if let Some(known) = self.cache.get(a) {
                    return *known;
                }
                let known = match self.body(a) {
                    Some(body) => self.known(&body),
                    None => None,
                }
                .or_else(|| {
                    self.widths
                        .get(a)
                        .filter(|w| **w <= 128)
                        .map(|w| Known::unknown(*w))
                });
                if self.work > 0 {
                    self.cache.insert(a.clone(), known);
                }
                known
            }
            Node::List(items) => self.known_list(node, items),
        }
    }

    fn known_list(&mut self, node: &Node, items: &[Node]) -> Option<Known> {
        if let Some(literal) =
            crate::heap::literal(&render(node)).filter(|_| head(node) == Some("_"))
        {
            let Node::Atom(w) = items.get(2)? else {
                return None;
            };
            let width: u32 = w.parse().ok()?;
            return (width <= 128).then(|| Known::exact(literal, width));
        }
        // Indexed operators: `((_ zero_extend k) x)` and friends.
        if let (Some(Node::List(op)), Some(x)) = (items.first(), items.get(1)) {
            let words: Vec<&str> = op
                .iter()
                .filter_map(|n| {
                    if let Node::Atom(a) = n {
                        Some(a.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let x = self.known(x)?;
            let number = |i: usize| words.get(i).and_then(|w| w.parse::<u32>().ok());
            return match words.get(1).copied() {
                Some("zero_extend") => {
                    let width = x.width.checked_add(number(2)?).filter(|w| *w <= 128)?;
                    Some(
                        Known::bits(x.zero | (mask(width) & !mask(x.width)), x.one, width)
                            .within(x.lo, x.hi),
                    )
                }
                Some("sign_extend") => {
                    let width = x.width.checked_add(number(2)?).filter(|w| *w <= 128)?;
                    let (sign, high) = (1u128 << (x.width - 1), mask(width) & !mask(x.width));
                    Some(Known::bits(
                        x.zero | if x.zero & sign != 0 { high } else { 0 },
                        x.one | if x.one & sign != 0 { high } else { 0 },
                        width,
                    ))
                }
                Some("extract") => {
                    let (high, low) = (number(2)?, number(3)?);
                    let width = high.checked_sub(low)? + 1;
                    Some(Known::bits(
                        (x.zero >> low) & mask(width),
                        (x.one >> low) & mask(width),
                        width,
                    ))
                }
                _ => None,
            };
        }
        let op = head(node)?;
        let args = &items[1..];
        match op {
            "bvnot" => Some(self.known(args.first()?)?.not()),
            "bvneg" => {
                let x = self.known(args.first()?)?;
                Some(add_carry(x.not(), Known::exact(0, x.width), false, true))
            }
            "concat" => {
                let mut acc: Option<Known> = None;
                for arg in args {
                    let k = self.known(arg)?;
                    acc = Some(match acc {
                        None => k,
                        Some(a) => {
                            let width = a.width + k.width;
                            if width > 128 {
                                return None;
                            }
                            Known::bits(
                                (a.zero << k.width) | k.zero,
                                (a.one << k.width) | k.one,
                                width,
                            )
                        }
                    });
                }
                acc
            }
            "bvand" | "bvor" | "bvxor" | "bvadd" | "bvmul" => {
                let mut known: Vec<Option<Known>> = Vec::new();
                for arg in args {
                    known.push(self.known(arg));
                }
                let width = known.iter().flatten().map(|k| k.width).next()?;
                let mut acc: Option<Known> = None;
                for k in known {
                    let k = k.unwrap_or(Known::unknown(width));
                    if k.width != width {
                        return None;
                    }
                    acc = Some(match acc {
                        None => k,
                        Some(a) => combine(op, a, k),
                    });
                }
                acc
            }
            "bvsub" | "bvshl" | "bvlshr" | "bvashr" | "bvurem" | "bvudiv" => {
                let (a, b) = (self.known(args.first()?), self.known(args.get(1)?));
                let width = a.or(b)?.width;
                let a = a.unwrap_or(Known::unknown(width));
                let b = b.unwrap_or(Known::unknown(width));
                Some(match (op, b.value()) {
                    ("bvsub", _) => {
                        let k = add_carry(a, b.not(), false, true);
                        if a.lo >= b.hi {
                            k.within(a.lo - b.hi, a.hi - b.lo)
                        } else {
                            k
                        }
                    }
                    ("bvshl" | "bvlshr" | "bvashr", Some(c)) => shift(op, a, c),
                    ("bvurem", Some(0)) | ("bvudiv", Some(1)) => a,
                    // A power of two keeps the low bits: `x & (c - 1)`.
                    ("bvurem", Some(c)) if c.is_power_of_two() => {
                        combine("bvand", a, Known::exact(c - 1, width)).within(0, a.hi)
                    }
                    ("bvurem", Some(c)) => {
                        let bits = 128 - (c - 1).leading_zeros();
                        Known::bits(mask(width) & !mask(bits), 0, width).within(0, a.hi.min(c - 1))
                    }
                    _ => match (a.value(), b.value()) {
                        (Some(x), Some(y)) if op == "bvudiv" && y != 0 => {
                            Known::exact(x / y, width)
                        }
                        _ => Known::unknown(width),
                    },
                })
            }
            "ite" => {
                let condition = self.boolean(args.first()?);
                match condition {
                    Node::Atom(c) if c == "true" => self.known(args.get(1)?),
                    Node::Atom(c) if c == "false" => self.known(args.get(2)?),
                    _ => {
                        let (a, b) = (self.known(args.get(1)?)?, self.known(args.get(2)?)?);
                        (a.width == b.width).then(|| {
                            Known::bits(a.zero & b.zero, a.one & b.one, a.width)
                                .within(a.lo.min(b.lo), a.hi.max(b.hi))
                        })
                    }
                }
            }
            "select" => {
                let value = self.read(args.first()?, args.get(1)?)?;
                self.known(&value)
            }
            _ => None,
        }
    }
}

fn combine(op: &str, a: Known, b: Known) -> Known {
    let w = a.width;
    match op {
        "bvand" => Known::bits(a.zero | b.zero, a.one & b.one, w).within(0, a.hi.min(b.hi)),
        "bvor" => Known::bits(a.zero & b.zero, a.one | b.one, w),
        "bvxor" => {
            let known = (a.zero | a.one) & (b.zero | b.one);
            let value = a.one ^ b.one;
            Known::bits(!value & known, value & known, w)
        }
        "bvadd" => {
            let k = add_carry(a, b, true, false);
            match a.hi.checked_add(b.hi).filter(|s| *s <= mask(w)) {
                Some(hi) => k.within(a.lo + b.lo + 1, hi),
                None => k,
            }
        }
        _ => match (a.value(), b.value()) {
            (Some(x), Some(y)) => Known::exact(x.wrapping_mul(y), w),
            _ => Known::bits(mask((a.trailing_zeros() + b.trailing_zeros()).min(w)), 0, w),
        },
    }
}

mod rewrite;

#[cfg(test)]
mod tests;

/// Phage self-check (tools/selfcheck.py): transfer functions are sound for
/// every 8-bit input, bounds included.
pub mod selfcheck {
    use super::*;
    fn contains(k: Known, v: u128) -> bool {
        v & k.zero == 0 && v & k.one == k.one && k.lo <= v && v <= k.hi && v <= mask(k.width)
    }
    fn input(zero: u8, one: u8, lo: u8, hi: u8, v: u8) -> Option<Known> {
        if zero & one != 0 {
            return None;
        }
        let k = Known::bits(u128::from(zero), u128::from(one), 8)
            .within(u128::from(lo), u128::from(hi));
        contains(k, u128::from(v)).then_some(k)
    }
    pub fn check(op: u8, a: [u8; 5], b: [u8; 5]) -> bool {
        let (Some(ka), Some(kb)) = (
            input(a[0], a[1], a[2], a[3], a[4]),
            input(b[0], b[1], b[2], b[3], b[4]),
        ) else {
            return true;
        };
        let (x, y) = (a[4], b[4]);
        let (k, v) = match op {
            0 => (combine("bvand", ka, kb), x & y),
            1 => (combine("bvor", ka, kb), x | y),
            2 => (combine("bvxor", ka, kb), x ^ y),
            3 => (combine("bvadd", ka, kb), x.wrapping_add(y)),
            4 => (combine("bvmul", ka, kb), x.wrapping_mul(y)),
            5 => (ka.not(), !x),
            6 => (add_carry(ka, kb.not(), false, true), x.wrapping_sub(y)),
            7 => (add_carry(ka.not(), Known::exact(0, 8), false, true), x.wrapping_neg()),
            8 => (shift("bvshl", ka, u128::from(y)), if y < 8 { x << y } else { 0 }),
            9 => (shift("bvlshr", ka, u128::from(y)), if y < 8 { x >> y } else { 0 }),
            10 => (shift("bvashr", ka, u128::from(y)), ((x as i8) >> y.min(7)) as u8),
            _ => return true,
        };
        contains(k, u128::from(v))
    }
}
