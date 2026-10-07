//! Rewrites on top of the known-bits analysis (knownbits.rs): `select`
//! over store chains, bit-vector literals and identities, and Boolean
//! conditions (connectives, comparisons decided by known bits or bounds,
//! and the `shl` flag identities).

use super::{Known, Node, Simplifier, atom, head, mask, parse};
use std::rc::Rc;

/// Stores walked when resolving one `select`.
const STORES: usize = 2048;

impl Simplifier<'_> {
    /// The value `(select array index)` reads, when the store chain decides
    /// it for every index value the index's known bits allow: each possible
    /// index is either overwritten by a literal-index store or may read a
    /// store at a symbolic index or the constant base, and all of those
    /// candidate values must be the same term.
    pub(super) fn read(&mut self, array: &Node, index: &Node) -> Option<Node> {
        let range = self.known(index).map(|k| (k.min(), k.max()));
        // Indices still unresolved: a bit per index of a small range, or
        // `None` for "any index" when the range is large or unknown.
        let mut open: Option<u64> = match range {
            Some((lo, hi)) if hi - lo < 64 => Some(if hi - lo == 63 {
                u64::MAX
            } else {
                (1u64 << (hi - lo + 1)) - 1
            }),
            _ => None,
        };
        let mut common: Option<Node> = None;
        let agree = |value: &Node, common: &mut Option<Node>| match common {
            None => {
                *common = Some(value.clone());
                true
            }
            Some(c) => c == value,
        };
        let mut root = Rc::new(array.clone());
        let mut stores = 0;
        loop {
            // Walk this body's store chain by reference; a name continues
            // in its own (cached) body.
            let mut node: &Node = &root;
            let name = loop {
                stores += 1;
                if stores > STORES || !self.tick() {
                    return None;
                }
                let Node::List(items) = node else {
                    let Node::Atom(name) = node else { return None };
                    break name.clone();
                };
                if head(node) != Some("store") {
                    // `((as const (Array ...)) value)` serves every open index.
                    let is_const = matches!(items.first(), Some(n) if head(n) == Some("as"));
                    let value = items.get(1).filter(|_| is_const && items.len() == 2)?;
                    return (open == Some(0) || agree(value, &mut common))
                        .then_some(common)
                        .flatten();
                }
                let [_, base, at, value] = items.as_slice() else {
                    return None;
                };
                if at == index {
                    // The very index: this store answers every open index.
                    return agree(value, &mut common).then_some(common).flatten();
                }
                let stored = self.known(at).map(|k| (k.min(), k.max()));
                let hits = match (range, stored) {
                    (Some((lo, hi)), Some((a, b))) if a > hi || b < lo => false,
                    (Some((lo, hi)), Some((a, b))) => match open {
                        Some(bits) => {
                            let (from, to) = (a.max(lo) - lo, b.min(hi) - lo);
                            let span = if to - from == 63 {
                                u64::MAX
                            } else {
                                ((1u64 << (to - from + 1)) - 1) << from
                            };
                            let hit = bits & span;
                            if a == b {
                                // A literal index resolves it for good.
                                open = Some(bits & !hit);
                            }
                            hit != 0
                        }
                        None => true,
                    },
                    _ => true,
                };
                if hits && !agree(value, &mut common) {
                    return None;
                }
                if open == Some(0) {
                    return common;
                }
                node = base;
            };
            root = self.body(&name)?;
        }
    }

    /// A bit-vector term with its fully known value as a literal.
    pub fn literal(&mut self, node: &Node) -> Node {
        match self
            .known(node)
            .and_then(|k| k.value().map(|v| (v, k.width)))
        {
            Some((v, w)) => parse(&crate::value::bv(v, w)).unwrap_or_else(|| node.clone()),
            None => node.clone(),
        }
    }

    /// A bit-vector term: a literal when fully known, else with decided
    /// `ite`s and identity operands (`x + 0`, `x | 0`, `x ^ 0`, `x & ~0`)
    /// removed at the top.
    pub fn bitvector(&mut self, node: &Node) -> Node {
        let literal = self.literal(node);
        if literal != *node || !self.tick() {
            return literal;
        }
        let Node::List(items) = node else {
            return node.clone();
        };
        match (head(node), items.len()) {
            (Some("ite"), 4) => match self.boolean(&items[1]) {
                Node::Atom(c) if c == "true" => self.bitvector(&items[2]),
                Node::Atom(c) if c == "false" => self.bitvector(&items[3]),
                _ => node.clone(),
            },
            (Some(op @ ("bvadd" | "bvor" | "bvxor" | "bvand")), 3) => {
                let identity = |k: Option<Known>| {
                    k.and_then(|k| k.value().map(|v| (v, k.width)))
                        .is_some_and(|(v, w)| if op == "bvand" { v == mask(w) } else { v == 0 })
                };
                if identity(self.known(&items[2])) {
                    self.bitvector(&items[1])
                } else if identity(self.known(&items[1])) {
                    self.bitvector(&items[2])
                } else {
                    node.clone()
                }
            }
            _ => node.clone(),
        }
    }

    /// A Boolean term rewritten bottom-up.
    pub fn boolean(&mut self, node: &Node) -> Node {
        if !self.tick() {
            return node.clone();
        }
        let Node::List(items) = node else {
            return node.clone();
        };
        let Some(op) = head(node) else {
            return node.clone();
        };
        let args = &items[1..];
        let truth = |b: bool| atom(if b { "true" } else { "false" });
        match op {
            "and" | "or" => {
                let (unit, zero) = if op == "and" {
                    ("true", "false")
                } else {
                    ("false", "true")
                };
                let mut kept: Vec<Node> = Vec::new();
                for arg in args {
                    let arg = self.boolean(arg);
                    match &arg {
                        Node::Atom(a) if a == zero => return atom(zero),
                        Node::Atom(a) if a == unit => {}
                        _ if kept.contains(&arg) => {}
                        _ => kept.push(arg),
                    }
                }
                match kept.len() {
                    0 => atom(unit),
                    1 => kept.pop().unwrap_or_else(|| atom(unit)),
                    _ => Node::List(std::iter::once(atom(op)).chain(kept).collect()),
                }
            }
            "not" => match self.boolean(args.first().unwrap_or(node)) {
                Node::Atom(a) if a == "true" => truth(false),
                Node::Atom(a) if a == "false" => truth(true),
                // Double negation, also through one defined name.
                Node::List(inner) if head(&Node::List(inner.clone())) == Some("not") => {
                    inner.get(1).cloned().unwrap_or_else(|| node.clone())
                }
                Node::Atom(name) => match self.body(&name) {
                    Some(body) if head(&body) == Some("not") => match body.as_ref() {
                        Node::List(inner) if inner.len() == 2 => inner[1].clone(),
                        _ => Node::List(vec![atom("not"), Node::Atom(name)]),
                    },
                    _ => Node::List(vec![atom("not"), Node::Atom(name)]),
                },
                inner => Node::List(vec![atom("not"), inner]),
            },
            "ite" if args.len() == 3 => match self.boolean(&args[0]) {
                Node::Atom(c) if c == "true" => self.boolean(&args[1]),
                Node::Atom(c) if c == "false" => self.boolean(&args[2]),
                c => Node::List(vec![
                    atom("ite"),
                    c,
                    self.boolean(&args[1]),
                    self.boolean(&args[2]),
                ]),
            },
            "select" if args.len() == 2 => match self.read(&args[0], &args[1]) {
                Some(value) => self.boolean(&value),
                None => node.clone(),
            },
            "=" if args.len() == 2 => {
                if let Some(decided) = self.shift_identity(&args[0], &args[1]) {
                    return truth(decided);
                }
                match (self.known(&args[0]), self.known(&args[1])) {
                    (Some(a), Some(b)) if a.width == b.width => {
                        if a.one & b.zero != 0 || a.zero & b.one != 0 || a.hi < b.lo || b.hi < a.lo
                        {
                            truth(false)
                        } else if a.value().is_some() && a.value() == b.value() {
                            truth(true)
                        } else {
                            Node::List(vec![
                                atom("="),
                                self.literal(&args[0]),
                                self.literal(&args[1]),
                            ])
                        }
                    }
                    (None, None) => {
                        let (a, b) = (self.boolean(&args[0]), self.boolean(&args[1]));
                        match (&a, &b) {
                            (Node::Atom(x), Node::Atom(y))
                                if matches!(x.as_str(), "true" | "false")
                                    && matches!(y.as_str(), "true" | "false") =>
                            {
                                truth(x == y)
                            }
                            _ if a == b => truth(true),
                            _ => Node::List(vec![atom("="), a, b]),
                        }
                    }
                    _ => node.clone(),
                }
            }
            "bvslt" | "bvsle" | "bvsgt" | "bvsge" if args.len() == 2 => {
                let (a, b) = match op {
                    "bvsgt" | "bvsge" => (&args[1], &args[0]),
                    _ => (&args[0], &args[1]),
                };
                let strict = matches!(op, "bvslt" | "bvsgt");
                // Flipping the sign bit maps signed order onto unsigned order.
                if let (Some(x), Some(y)) = (self.known(a), self.known(b))
                    && x.width == y.width
                {
                    let (x, y) = (flip_sign(x), flip_sign(y));
                    let always = if strict {
                        x.max() < y.min()
                    } else {
                        x.max() <= y.min()
                    };
                    let never = if strict {
                        x.min() >= y.max()
                    } else {
                        x.min() > y.max()
                    };
                    if always || never {
                        return truth(always);
                    }
                }
                Node::List(vec![
                    atom(op),
                    self.literal(&args[0]),
                    self.literal(&args[1]),
                ])
            }
            "bvult" | "bvule" | "bvugt" | "bvuge" if args.len() == 2 => {
                let (a, b) = match op {
                    "bvugt" | "bvuge" => (&args[1], &args[0]),
                    _ => (&args[0], &args[1]),
                };
                let strict = matches!(op, "bvult" | "bvugt");
                // `a < b` (or `a <= b`), from the ranges known bits allow.
                if let (Some(x), Some(y)) = (self.known(a), self.known(b))
                    && x.width == y.width
                {
                    let always = if strict {
                        x.max() < y.min()
                    } else {
                        x.max() <= y.min()
                    };
                    let never = if strict {
                        x.min() >= y.max()
                    } else {
                        x.min() > y.max()
                    };
                    if always {
                        return truth(true);
                    }
                    if never {
                        return truth(false);
                    }
                }
                // `nuw` addition: `(bvuge (bvadd p q) p)` holds without overflow.
                if op == "bvuge"
                    && head(&args[0]) == Some("bvadd")
                    && let Node::List(sum) = &args[0]
                    && sum.len() == 3
                    && (sum[1] == args[1] || sum[2] == args[1])
                    && let (Some(p), Some(q)) = (self.known(&sum[1]), self.known(&sum[2]))
                    && p.max()
                        .checked_add(q.max())
                        .is_some_and(|s| s <= mask(p.width))
                {
                    return truth(true);
                }
                Node::List(vec![
                    atom(op),
                    self.literal(&args[0]),
                    self.literal(&args[1]),
                ])
            }
            _ => node.clone(),
        }
    }

    /// `nuw`/`nsw` on `shl`: `(= (bvlshr (bvshl x c) c) x)` holds when the top
    /// `c` bits of `x` are known zero; with `bvashr`, when the top `c + 1`
    /// bits are all known zero or all known one.
    fn shift_identity(&mut self, left: &Node, right: &Node) -> Option<bool> {
        let Node::List(outer) = left else { return None };
        let [Node::Atom(op), Node::List(inner), amount] = outer.as_slice() else {
            return None;
        };
        let [Node::Atom(shl), x, amount2] = inner.as_slice() else {
            return None;
        };
        if shl != "bvshl"
            || amount != amount2
            || x != right
            || !matches!(op.as_str(), "bvlshr" | "bvashr")
        {
            return None;
        }
        let k = self.known(x)?;
        let c = u32::try_from(self.known(amount)?.value()?).ok()?;
        let top = c.checked_add(u32::from(op == "bvashr"))?;
        if top > k.width {
            return None;
        }
        let high = mask(k.width) & !mask(k.width - top);
        (k.zero & high == high || (op == "bvashr" && k.one & high == high)).then_some(true)
    }
}

/// `x` with its sign bit inverted: signed order of `x` is unsigned order of
/// the result. Bounds shift by half the range when the sign bit is known.
fn flip_sign(x: Known) -> Known {
    let sign = 1u128 << (x.width - 1);
    let bits = Known::bits(
        (x.zero & !sign) | (x.one & sign),
        (x.one & !sign) | (x.zero & sign),
        x.width,
    );
    if x.zero & sign != 0 {
        bits.within(x.lo + sign, x.hi + sign)
    } else if x.one & sign != 0 {
        bits.within(x.lo - sign, x.hi - sign)
    } else {
        bits
    }
}
