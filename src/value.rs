//! Symbolic scalar and pointer values with explicit definedness. LLVM
//! poison travels through computations and is checked at observable uses.

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Bits(u32),
    Undef(u32),
    Pointer {
        object: String,
        offset: String,
        bounded: bool,
    },
    PointerChoice {
        condition: String,
        yes: Box<Value>,
        no: Box<Value>,
    },
    Aggregate(Vec<Value>),
    /// An `i64` from ptrtoint or an exact memory copy of this pointer. Storing it moves the pointer's provenance (a struct
    /// copied through integers); any computation on it drops that.
    PointerBits(Box<Value>),
    /// An integer read from memory with per-byte definedness (see
    /// partial.rs): `flags[i]` is byte i's, `poison` the whole value's.
    Bytes {
        flags: Vec<String>,
        poison: String,
    },
    /// An IEEE-754 value as an SMT FloatingPoint term (exponent and
    /// significand bits; floats.rs).
    Float(u32, u32),
    /// A `select` with one undef arm: undef (any value at each use) when
    /// `condition` holds, else `other`. Observed like undef (operands.rs).
    UndefChoice {
        condition: String,
        other: Box<Value>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub expr: String,
    pub kind: Kind,
    pub defined: String,
}

pub fn bv(value: u128, width: u32) -> String {
    format!("(_ bv{value} {width})")
}
pub fn and(items: &[String]) -> String {
    if items.iter().any(|s| s == "false") {
        return "false".into();
    }
    let items: Vec<_> = items
        .iter()
        .filter(|s| s.as_str() != "true")
        .cloned()
        .collect();
    if items.is_empty() {
        "true".into()
    } else if items.len() == 1 {
        items[0].clone()
    } else {
        format!("(and {})", items.join(" "))
    }
}
pub fn or(items: &[String]) -> String {
    if items.iter().any(|s| s == "true") {
        return "true".into();
    }
    let items: Vec<_> = items
        .iter()
        .filter(|s| s.as_str() != "false")
        .cloned()
        .collect();
    match items.len() {
        0 => "false".into(),
        1 => items[0].clone(),
        _ => format!("(or {})", items.join(" ")),
    }
}
pub fn not(value: &str) -> String {
    match value {
        "true" => "false".into(),
        "false" => "true".into(),
        _ => format!("(not {value})"),
    }
}
pub fn truth(value: &Value) -> Result<String, String> {
    if value.width()? != 1 {
        return Err("expected a one-bit condition".into());
    }
    if value.expr == bv(1, 1) {
        return Ok("true".into());
    }
    if value.expr == bv(0, 1) {
        return Ok("false".into());
    }
    Ok(format!("(= {} {})", value.expr, bv(1, 1)))
}

impl Value {
    pub fn bits(expr: String, width: u32, defined: String) -> Self {
        Self {
            expr,
            kind: Kind::Bits(width),
            defined,
        }
    }
    pub fn width(&self) -> Result<u32, String> {
        match self.kind {
            Kind::Bits(w) => Ok(w),
            Kind::PointerBits(_) => Ok(64),
            Kind::Bytes { ref flags, .. } => Ok(8 * flags.len() as u32),
            Kind::Undef(width) => Err(format!("observable LLVM undef i{width} is unsupported")),
            Kind::UndefChoice { .. } => Err("observable LLVM undef choice is unsupported".into()),
            _ => Err("expected integer value".into()),
        }
    }
    pub fn pointer(&self) -> Result<(&str, &str), String> {
        match &self.kind {
            Kind::Pointer { object, offset, .. } => Ok((object, offset)),
            _ => Err("expected an object-relative pointer".into()),
        }
    }
}

pub fn constant(text: &str, width: u32) -> Result<Value, String> {
    if width == 0 || width > 128 {
        return Err("integer width must be within 1..128".into());
    }
    let value = match text {
        "true" => 1,
        "false" => 0,
        _ if text.starts_with('-') => {
            let magnitude = text[1..]
                .parse::<u128>()
                .map_err(|_| format!("invalid integer {text}"))?;
            let mask = if width == 128 {
                u128::MAX
            } else {
                (1u128 << width) - 1
            };
            magnitude.wrapping_neg() & mask
        }
        _ => text
            .parse::<u128>()
            .map_err(|_| format!("unsupported constant {text}"))?,
    };
    // LLVM constants are taken modulo 2^width, as SMT-LIB literals are.
    let mask = if width == 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    };
    Ok(Value::bits(bv(value & mask, width), width, "true".into()))
}

pub fn binary(op: &str, flags: &[&str], a: &Value, b: &Value) -> Result<Value, String> {
    let width = a.width()?;
    if b.width()? != width {
        return Err("integer operand widths differ".into());
    }
    let smt = match op {
        "add" => "bvadd",
        "sub" => "bvsub",
        "mul" => "bvmul",
        "and" => "bvand",
        "or" => "bvor",
        "xor" => "bvxor",
        "shl" => "bvshl",
        "lshr" => "bvlshr",
        "ashr" => "bvashr",
        "udiv" => "bvudiv",
        "urem" => "bvurem",
        "sdiv" => "bvsdiv",
        "srem" => "bvsrem",
        _ => return Err(format!("unsupported binary operator {op}")),
    };
    let expr = format!("({smt} {} {})", a.expr, b.expr);
    let mut valid = vec![a.defined.clone(), b.defined.clone()];
    if matches!(op, "shl" | "lshr" | "ashr") {
        valid.push(format!(
            "(bvult {} {})",
            b.expr,
            bv(u128::from(width), width)
        ));
    }
    if matches!(op, "udiv" | "urem" | "sdiv" | "srem") {
        valid.push(format!("(not (= {} {}))", b.expr, bv(0, width)));
    }
    if matches!(op, "sdiv" | "srem") {
        valid.push(signed_division_defined(a, b, width));
    }
    for flag in flags {
        match (*flag, op) {
            ("nuw", "add") => valid.push(format!("(bvuge {expr} {})", a.expr)),
            ("nuw", "sub") => valid.push(format!("(bvuge {} {})", a.expr, b.expr)),
            ("nuw", "shl") => valid.push(format!("(= (bvlshr {expr} {}) {})", b.expr, a.expr)),
            ("nsw", "shl") => valid.push(format!("(= (bvashr {expr} {}) {})", b.expr, a.expr)),
            ("exact", "udiv") => valid.push(format!(
                "(= (bvurem {} {}) {})",
                a.expr,
                b.expr,
                bv(0, width)
            )),
            ("exact", "sdiv") => valid.push(format!(
                "(= (bvsrem {} {}) {})",
                a.expr,
                b.expr,
                bv(0, width)
            )),
            ("exact", "lshr" | "ashr") => {
                valid.push(format!("(= (bvshl {expr} {}) {})", b.expr, a.expr))
            }
            ("disjoint", "or") => valid.push(format!(
                "(= (bvand {} {}) {})",
                a.expr,
                b.expr,
                bv(0, width)
            )),
            ("nuw" | "nsw", "mul") => {
                let extend = if *flag == "nuw" {
                    "zero_extend"
                } else {
                    "sign_extend"
                };
                valid.push(format!("(= (bvmul ((_ {extend} {width}) {}) ((_ {extend} {width}) {})) ((_ {extend} {width}) {expr}))",a.expr,b.expr));
            }
            ("nsw", "add" | "sub") => {
                let bit = width - 1;
                let sa = format!("((_ extract {bit} {bit}) {})", a.expr);
                let sb = format!("((_ extract {bit} {bit}) {})", b.expr);
                let sr = format!("((_ extract {bit} {bit}) {expr})");
                let relation = if op == "add" {
                    format!("(= {sa} {sb})")
                } else {
                    format!("(not (= {sa} {sb}))")
                };
                valid.push(format!("(not (and {relation} (not (= {sr} {sa}))))"));
            }
            _ => return Err(format!("unsupported flag {flag} on {op}")),
        }
    }
    Ok(Value::bits(expr, width, and(&valid)))
}

/// LLVM sdiv/srem: INT_MIN divided by -1 overflows, which is immediate UB.
pub fn signed_division_defined(a: &Value, b: &Value, width: u32) -> String {
    format!(
        "(not (and (= {} {}) (= {} {})))",
        a.expr,
        bv(1 << (width - 1), width),
        b.expr,
        bv(u128::MAX >> (128 - width), width)
    )
}

pub fn compare(predicate: &str, same_sign: bool, a: &Value, b: &Value) -> Result<Value, String> {
    let mut valid = vec![a.defined.clone(), b.defined.clone()];
    if matches!(predicate, "eq" | "ne") && !same_sign {
        if let Kind::PointerChoice { condition, yes, no } = &a.kind {
            let yes = compare(predicate, false, yes, b)?;
            let no = compare(predicate, false, no, b)?;
            return Ok(Value::bits(
                format!("(ite {condition} {} {})", yes.expr, no.expr),
                1,
                and(&valid),
            ));
        }
        if matches!(b.kind, Kind::PointerChoice { .. }) {
            return compare(predicate, false, b, a);
        }
        if let (
            Kind::Pointer {
                object: ao,
                bounded: ab,
                ..
            },
            Kind::Pointer {
                object: bo,
                bounded: bb,
                ..
            },
        ) = (&a.kind, &b.kind)
            && (ao.is_empty() && bo.is_empty() || (ao.is_empty() && *bb) || (bo.is_empty() && *ab))
        {
            // A defined pointer within a nonnull, nonwrapping object
            // cannot be null. Poison obligations remain in definedness.
            let equal = ao.is_empty() && bo.is_empty();
            return Ok(Value::bits(
                bv(u128::from(equal == (predicate == "eq")), 1),
                1,
                and(&valid),
            ));
        }
    }
    let (left, right, width) = match (&a.kind, &b.kind) {
        (
            Kind::Pointer {
                object: ao,
                offset: ap,
                ..
            },
            Kind::Pointer {
                object: bo,
                offset: bp,
                ..
            },
        ) if ao == bo && matches!(predicate, "eq" | "ne") => {
            // Equal bases cancel in modular address equality. Ordered
            // comparisons still use full addresses because they may wrap.
            (ap.clone(), bp.clone(), 64)
        }
        (
            Kind::Pointer { .. } | Kind::PointerChoice { .. },
            Kind::Pointer { .. } | Kind::PointerChoice { .. },
        ) => (a.expr.clone(), b.expr.clone(), 64),
        // Pointer bytes read without provenance compare by address.
        (
            Kind::Bytes { .. } | Kind::Bits(64),
            Kind::Pointer { .. } | Kind::PointerChoice { .. },
        )
        | (
            Kind::Pointer { .. } | Kind::PointerChoice { .. },
            Kind::Bytes { .. } | Kind::Bits(64),
        ) if a.width().unwrap_or(64) == 64 && b.width().unwrap_or(64) == 64 => {
            (a.expr.clone(), b.expr.clone(), 64)
        }
        (
            Kind::Bits(_) | Kind::PointerBits(_) | Kind::Bytes { .. },
            Kind::Bits(_) | Kind::PointerBits(_) | Kind::Bytes { .. },
        ) if a.width()? == b.width()? => (a.expr.clone(), b.expr.clone(), a.width()?),
        _ => return Err("comparison types differ".into()),
    };
    if same_sign {
        let bit = width - 1;
        valid.push(format!(
            "(= ((_ extract {bit} {bit}) {left}) ((_ extract {bit} {bit}) {right}))"
        ));
    }
    let expr = match predicate {
        "eq" => format!("(= {left} {right})"),
        "ne" => format!("(not (= {left} {right}))"),
        "ult" | "ule" | "ugt" | "uge" | "slt" | "sle" | "sgt" | "sge" => {
            format!("(bv{predicate} {left} {right})")
        }
        _ => return Err(format!("unsupported comparison {predicate}")),
    };
    Ok(Value::bits(
        format!("(ite {expr} {} {})", bv(1, 1), bv(0, 1)),
        1,
        and(&valid),
    ))
}

/// `select`: bits become `ite`, pointers a resolvable choice, aggregates
/// choose field by field. A poison condition poisons every leaf.
pub fn choose(condition: &str, defined: &str, yes: Value, no: Value) -> Result<Value, String> {
    let leaf_defined = |y: &Value, n: &Value| {
        and(&[
            defined.to_owned(),
            format!("(ite {condition} {} {})", y.defined, n.defined),
        ])
    };
    let expr = format!("(ite {condition} {} {})", yes.expr, no.expr);
    match (&yes.kind, &no.kind) {
        // Choosing unchanged pointer address bits chooses their provenance,
        // too. No arithmetic or unrelated integer acquires provenance here.
        (Kind::PointerBits(y), Kind::PointerBits(n)) => {
            let pointer = choose(condition, defined, (**y).clone(), (**n).clone())?;
            Ok(Value {
                expr,
                defined: leaf_defined(&yes, &no),
                kind: Kind::PointerBits(Box::new(pointer)),
            })
        }
        // Per-byte definedness is selected byte by byte.
        (Kind::Bytes { .. }, _) | (_, Kind::Bytes { .. }) if yes.width()? == no.width()? => {
            crate::partial::choose(condition, defined, expr, &yes, &no)
        }
        (
            Kind::Bits(_) | Kind::PointerBits(_) | Kind::Bytes { .. },
            Kind::Bits(_) | Kind::PointerBits(_) | Kind::Bytes { .. },
        ) if yes.width()? == no.width()? => {
            Ok(Value::bits(expr, yes.width()?, leaf_defined(&yes, &no)))
        }
        (Kind::Float(e, s), Kind::Float(f, t)) if (e, s) == (f, t) => Ok(Value {
            expr,
            kind: Kind::Float(*e, *s),
            defined: leaf_defined(&yes, &no),
        }),
        // Two pointers into one object: a single pointer with a chosen offset,
        // which later accesses can use without resolving the condition.
        (
            Kind::Pointer {
                object: a,
                offset: x,
                bounded: bx,
            },
            Kind::Pointer {
                object: b,
                offset: y,
                bounded: by,
            },
        ) if a == b && !a.is_empty() => Ok(Value {
            expr,
            defined: leaf_defined(&yes, &no),
            kind: Kind::Pointer {
                object: a.clone(),
                offset: format!("(ite {condition} {x} {y})"),
                bounded: *bx && *by,
            },
        }),
        (
            Kind::Pointer { .. } | Kind::PointerChoice { .. } | Kind::Bytes { .. },
            Kind::Pointer { .. } | Kind::PointerChoice { .. } | Kind::Bytes { .. },
        ) => Ok(Value {
            expr,
            defined: leaf_defined(&yes, &no),
            kind: Kind::PointerChoice {
                condition: condition.to_owned(),
                yes: Box::new(yes),
                no: Box::new(no),
            },
        }),
        (Kind::Aggregate(a), Kind::Aggregate(b)) if a.len() == b.len() => {
            let fields = a
                .iter()
                .zip(b)
                .map(|(y, n)| choose(condition, defined, y.clone(), n.clone()))
                .collect::<Result<_, _>>()?;
            Ok(Value {
                expr: String::new(),
                kind: Kind::Aggregate(fields),
                defined: "true".into(),
            })
        }
        _ => Err("select types differ".into()),
    }
}

/// A poison (or, with `zero`, zeroinitializer) value of a struct or array type.
pub fn aggregate(ty: &str, zero: bool) -> Result<Value, String> {
    let ty = ty.trim();
    let defined = if zero { "true" } else { "false" };
    if ty == "ptr" {
        return Ok(Value {
            expr: bv(0, 64),
            kind: Kind::Pointer {
                object: String::new(),
                offset: bv(0, 64),
                bounded: false,
            },
            defined: defined.into(),
        });
    }
    if let Some(width) = ty.strip_prefix('i').and_then(|w| w.parse::<u32>().ok()) {
        return Ok(Value::bits(bv(0, width), width, defined.into()));
    }
    let fields = if let Some(inner) = ty
        .strip_prefix("<{")
        .and_then(|t| t.strip_suffix("}>"))
        .or_else(|| ty.strip_prefix('{').and_then(|t| t.strip_suffix('}')))
    {
        crate::ir::split(inner)
            .into_iter()
            .map(|field| aggregate(field, zero))
            .collect::<Result<Vec<_>, _>>()?
    } else if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let (count, element) = inner.split_once(" x ").ok_or("array type malformed")?;
        let count: usize = count.trim().parse().map_err(|_| "array count malformed")?;
        if count > 256 {
            return Err("large aggregate constants are unsupported".into());
        }
        vec![aggregate(element, zero)?; count]
    } else {
        return Err(format!("unsupported aggregate field type {ty}"));
    };
    Ok(Value {
        expr: String::new(),
        kind: Kind::Aggregate(fields),
        defined: "true".into(),
    })
}

pub fn extract(value: Value, path: &[usize]) -> Result<Value, String> {
    let Some((first, rest)) = path.split_first() else {
        return Ok(value);
    };
    let Kind::Aggregate(fields) = value.kind else {
        return Err("extractvalue needs an aggregate".into());
    };
    let field = fields
        .into_iter()
        .nth(*first)
        .ok_or("aggregate index out of range")?;
    extract(field, rest)
}

pub fn insert(value: Value, path: &[usize], element: Value) -> Result<Value, String> {
    let Some((first, rest)) = path.split_first() else {
        return Ok(element);
    };
    let Kind::Aggregate(mut fields) = value.kind else {
        return Err("insertvalue needs an aggregate".into());
    };
    let field = fields
        .get_mut(*first)
        .ok_or("aggregate index out of range")?;
    *field = insert(field.clone(), rest, element)?;
    Ok(Value {
        kind: Kind::Aggregate(fields),
        ..value
    })
}

/// Definedness of every leaf, so a poison field of an aggregate counts.
pub fn fully_defined(value: &Value) -> String {
    match &value.kind {
        // An observed undef may be any value: never acceptable where noundef.
        Kind::Undef(_) => "false".into(),
        Kind::UndefChoice { condition, other } => and(&[not(condition), fully_defined(other)]),
        Kind::Aggregate(fields) => {
            let leaves: Vec<_> = fields.iter().map(fully_defined).collect();
            and(&[value.defined.clone(), and(&leaves)])
        }
        _ => value.defined.clone(),
    }
}
