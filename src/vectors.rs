//! Fixed-width integer vectors (`<N x iW>`), as SIMD library code such as
//! hashbrown's SSE2 group scans uses them. A vector value is an aggregate of
//! scalar lanes, so each lane carries its own poison: an `insertelement`
//! into `poison` followed by a broadcast shuffle is fully defined. Lane 0 is
//! the lowest-addressed and least significant, matching little-endian
//! memory and `bitcast`. Division, vector conditions and pointer lanes stay
//! unsupported.

use crate::{
    engine::{Engine, State},
    ir::split,
    memory,
    value::{self, Kind, Value, and, bv},
};

/// `(lanes, lane width)` of an integer vector type such as `<16 x i8>`.
pub fn vector_type(ty: &str) -> Option<(usize, u32)> {
    let inner = ty.trim().strip_prefix('<')?.strip_suffix('>')?;
    let (count, element) = inner.split_once(" x ")?;
    let width = element.trim().strip_prefix('i')?.parse::<u32>().ok()?;
    let count = count.trim().parse::<usize>().ok()?;
    (count > 0 && count <= 64 && (1..=128).contains(&width)).then_some((count, width))
}

/// Splits `[flags] <N x iW> operand` into flags, the vector type and the
/// operand text; `None` when the text has no vector type.
pub fn split_vector(text: &str) -> Option<(Vec<&str>, &str, &str)> {
    let text = text.trim();
    let open = text.find('<')?;
    let close = open + text[open..].find('>')?;
    let ty = &text[open..=close];
    vector_type(ty)?;
    Some((
        text[..open].split_whitespace().collect(),
        ty,
        text[close + 1..].trim(),
    ))
}

pub fn vector(lanes: Vec<Value>) -> Value {
    Value {
        expr: String::new(),
        kind: Kind::Aggregate(lanes),
        defined: "true".into(),
    }
}

fn poison_lane(width: u32) -> Value {
    Value::bits(bv(0, width), width, "false".into())
}

/// The lanes of a vector value, checked against the expected type.
pub fn lanes(value: &Value, count: usize, width: u32) -> Result<&[Value], String> {
    match &value.kind {
        Kind::Aggregate(lanes)
            if lanes.len() == count && lanes.iter().all(|l| l.width().ok() == Some(width)) =>
        {
            Ok(lanes)
        }
        _ => Err("vector operand has an unexpected shape".into()),
    }
}

/// A constant vector: `poison`, `undef` (treated as poison, which is
/// stricter), `zeroinitializer`, `splat (iW c)` or `<iW a, iW b, ...>`.
pub fn constant(text: &str, count: usize, width: u32) -> Result<Value, String> {
    let text = text.trim();
    let scalar = |item: &str| -> Result<Value, String> {
        let item = item.trim();
        let (ty, value) = item.split_once(' ').ok_or("vector lane type missing")?;
        if ty != format!("i{width}") {
            return Err("vector lane type differs".into());
        }
        match value.trim() {
            "poison" | "undef" => Ok(poison_lane(width)),
            literal => value::constant(literal, width),
        }
    };
    let lanes = match text {
        "poison" | "undef" => vec![poison_lane(width); count],
        "zeroinitializer" => vec![Value::bits(bv(0, width), width, "true".into()); count],
        _ if text.starts_with("splat (") => {
            let inner = text
                .strip_prefix("splat (")
                .and_then(|t| t.strip_suffix(')'))
                .ok_or("splat malformed")?;
            vec![scalar(inner)?; count]
        }
        _ if text.starts_with('<') => {
            let inner = text
                .strip_prefix('<')
                .and_then(|t| t.strip_suffix('>'))
                .ok_or("vector constant malformed")?;
            let lanes = split(inner)
                .into_iter()
                .map(scalar)
                .collect::<Result<Vec<_>, _>>()?;
            if lanes.len() != count {
                return Err("vector constant has the wrong lane count".into());
            }
            lanes
        }
        _ => return Err(format!("unsupported vector operand {text}")),
    };
    Ok(vector(lanes))
}

/// Reinterprets bits between integers and integer vectors of equal size.
/// A target lane is poison when any source lane it overlaps is.
pub fn bitcast(source: &Value, source_ty: &str, target_ty: &str) -> Result<Value, String> {
    let pieces: Vec<(String, u32, String)> = match vector_type(source_ty) {
        Some((count, width)) => lanes(source, count, width)?
            .iter()
            .flat_map(|l| {
                // A lane with per-byte definedness splits into its bytes.
                crate::partial::pieces(l)
                    .unwrap_or_else(|| vec![(l.expr.clone(), width, l.defined.clone())])
            })
            .collect(),
        None => crate::partial::pieces(source).unwrap_or_else(|| {
            vec![(
                source.expr.clone(),
                source.width().unwrap_or(0),
                source.defined.clone(),
            )]
        }),
    };
    if pieces.iter().any(|p| p.1 == 0) {
        return Err("bitcast of a non-integer value".into());
    }
    let total: u32 = pieces.iter().map(|p| p.1).sum();
    let (count, width, is_vector) = match vector_type(target_ty) {
        Some((count, width)) => (count, width, true),
        None => {
            let width = target_ty
                .trim()
                .strip_prefix('i')
                .and_then(|w| w.parse::<u32>().ok())
                .ok_or("only integer and vector bitcasts are supported")?;
            (1, width, false)
        }
    };
    if total != width * count as u32 {
        return Err("bitcast sizes differ".into());
    }
    let mut out = Vec::new();
    for lane in 0..count as u32 {
        let (low, high) = (lane * width, (lane + 1) * width);
        let mut parts = Vec::new();
        let mut defined = Vec::new();
        let mut start = 0;
        for (expr, bits, lane_defined) in &pieces {
            let (from, to) = (low.max(start), high.min(start + bits));
            if from < to {
                parts.push(if from == start && to == start + bits {
                    expr.clone()
                } else {
                    format!("((_ extract {} {}) {expr})", to - 1 - start, from - start)
                });
                defined.push(lane_defined.clone());
            }
            start += bits;
        }
        // Higher lanes are more significant.
        let expr = parts
            .into_iter()
            .reduce(|low, high| format!("(concat {high} {low})"))
            .ok_or("empty bitcast lane")?;
        out.push(Value::bits(expr, width, and(&defined)));
    }
    Ok(if is_vector {
        vector(out)
    } else {
        out.pop().ok_or("empty bitcast")?
    })
}

impl Engine {
    /// A lane pointer `lane * bytes` past `pointer`.
    fn lane_pointer(pointer: &Value, lane: usize, bytes: u64) -> Result<Value, String> {
        let (object, offset) = pointer.pointer()?;
        let step = bv(u128::from(lane as u64 * bytes), 64);
        Ok(Value {
            expr: format!("(bvadd {} {step})", pointer.expr),
            kind: Kind::Pointer {
                object: object.to_owned(),
                offset: format!("(bvadd {offset} {step})"),
                bounded: true,
            },
            defined: pointer.defined.clone(),
        })
    }

    /// `load <N x iW>, ptr P, align A`; undefined lanes are poison, and
    /// undefined behavior only with `!noundef`, as for the scalar load.
    pub fn vector_load(
        &mut self,
        state: &mut State,
        ty: &str,
        pointer: &Value,
        alignment: u64,
        noundef: bool,
    ) -> Result<Value, String> {
        let (count, width) = vector_type(ty).ok_or("vector load type")?;
        if !width.is_multiple_of(8) {
            return Err("vector lanes narrower than a byte are unsupported in memory".into());
        }
        let bytes = u64::from(width / 8);
        let valid = memory::access_load(&state.memory, pointer, bytes * count as u64, alignment)?;
        self.safety(state, &valid, "invalid memory read")?;
        let mut lanes = Vec::new();
        for lane in 0..count {
            let at = Self::lane_pointer(pointer, lane, bytes)?;
            lanes.push(memory::load(&state.memory, &at, width)?);
        }
        if noundef {
            let all: Vec<_> = lanes.iter().map(|l| l.defined.clone()).collect();
            self.safety(state, &and(&all), "uninitialized or poison memory read")?;
        }
        Ok(vector(lanes))
    }

    /// `store <N x iW> V, ptr P, align A`, lane by lane.
    pub fn vector_store(
        &mut self,
        state: &mut State,
        ty: &str,
        data: &Value,
        pointer: &Value,
        alignment: u64,
    ) -> Result<(), String> {
        let (count, width) = vector_type(ty).ok_or("vector store type")?;
        if !width.is_multiple_of(8) {
            return Err("vector lanes narrower than a byte are unsupported in memory".into());
        }
        let bytes = u64::from(width / 8);
        let valid = memory::access_store(&state.memory, pointer, bytes * count as u64, alignment)?;
        self.safety(state, &valid, "invalid memory write")?;
        let values = lanes(data, count, width)?.to_vec();
        for (lane, value) in values.iter().enumerate() {
            let at = Self::lane_pointer(pointer, lane, bytes)?;
            memory::store(&mut state.memory, &at, value)?;
        }
        Ok(())
    }

    /// Lane-wise binary operators and `icmp` on `[flags] <N x iW> a, b`.
    pub fn vector_lanewise(
        &mut self,
        state: &State,
        opcode: &str,
        args: &[&str],
    ) -> Result<Value, String> {
        let [left, right] = args else {
            return Err("vector operands missing".into());
        };
        let (words, ty, left) = split_vector(left).ok_or("vector type missing")?;
        let (count, width) = vector_type(ty).ok_or("vector type")?;
        let left = self.operand(state, left, ty)?;
        let right = self.operand(state, right, ty)?;
        let (left, right) = (lanes(&left, count, width)?, lanes(&right, count, width)?);
        let mut out = Vec::new();
        for (a, b) in left.iter().zip(right) {
            out.push(if opcode == "icmp" {
                let same = words.first() == Some(&"samesign");
                let predicate = words
                    .get(usize::from(same))
                    .ok_or("comparison predicate missing")?;
                value::compare(predicate, same, a, b)?
            } else if matches!(opcode, "udiv" | "urem" | "sdiv" | "srem") {
                return Err("vector division is unsupported".into());
            } else {
                value::binary(opcode, &words, a, b)?
            });
        }
        Ok(vector(out))
    }

    /// `insertelement`, `extractelement` and `shufflevector` with constant
    /// indices; an out-of-range index yields poison, as LangRef specifies.
    pub fn vector_element(
        &mut self,
        state: &State,
        opcode: &str,
        rest: &str,
    ) -> Result<Value, String> {
        let args = split(rest);
        let (_, ty, text) = split_vector(args.first().ok_or("vector operand missing")?)
            .ok_or("vector type missing")?;
        let (count, width) = vector_type(ty).ok_or("vector type")?;
        let base = self.operand(state, text, ty)?;
        let base = lanes(&base, count, width)?.to_vec();
        let index = |engine: &mut Self, text: &str| -> Result<Option<usize>, String> {
            let index = engine.typed_operand(state, text)?;
            if index.defined != "true" {
                return Err("a possibly poison vector index is unsupported".into());
            }
            let n = engine
                .concrete(state, &index.expr)?
                .ok_or("symbolic vector index is unsupported")?;
            Ok(usize::try_from(n).ok().filter(|n| *n < count))
        };
        match opcode {
            "extractelement" => {
                let at = index(self, args.get(1).ok_or("index missing")?)?;
                Ok(at.map_or_else(|| poison_lane(width), |i| base[i].clone()))
            }
            "insertelement" => {
                let element = self.typed_operand(state, args.get(1).ok_or("element missing")?)?;
                if element.width()? != width {
                    return Err("inserted element width differs".into());
                }
                let Some(at) = index(self, args.get(2).ok_or("index missing")?)? else {
                    return Ok(vector(vec![poison_lane(width); count]));
                };
                let mut lanes = base;
                lanes[at] = element;
                Ok(vector(lanes))
            }
            "shufflevector" => {
                let (_, other_ty, other) = split_vector(args.get(1).ok_or("vector missing")?)
                    .ok_or("vector type missing")?;
                if other_ty != ty {
                    return Err("shuffled vector types differ".into());
                }
                let other = self.operand(state, other, ty)?;
                let other = lanes(&other, count, width)?.to_vec();
                let (_, mask_ty, mask) =
                    split_vector(args.get(2).ok_or("mask missing")?).ok_or("mask type missing")?;
                let (length, mask_width) = vector_type(mask_ty).ok_or("mask type")?;
                let mask = constant(mask, length, mask_width)?;
                let mut out = Vec::new();
                for lane in lanes(&mask, length, mask_width)? {
                    let chosen = if lane.defined != "true" {
                        None
                    } else {
                        crate::heap::literal(&lane.expr)
                            .and_then(|m| usize::try_from(m).ok())
                            .and_then(|m| base.get(m).or_else(|| other.get(m.checked_sub(count)?)))
                    };
                    out.push(chosen.cloned().unwrap_or_else(|| poison_lane(width)));
                }
                Ok(vector(out))
            }
            _ => Err(format!("unsupported vector instruction {opcode}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitcasts_follow_lane_order() {
        let bytes = constant("<i8 1, i8 2, i8 3, i8 4>", 4, 8).unwrap();
        let word = bitcast(&bytes, "<4 x i8>", "i32").unwrap();
        assert_eq!(
            word.expr,
            "(concat (_ bv4 8) (concat (_ bv3 8) (concat (_ bv2 8) (_ bv1 8))))"
        );
        let halves = bitcast(&word, "i32", "<2 x i16>").unwrap();
        let halves = lanes(&halves, 2, 16).unwrap();
        assert!(halves[0].expr.contains("extract 15 0"));
        assert!(halves[1].expr.contains("extract 31 16"));
        // A poison lane poisons exactly the target lanes it overlaps.
        let mixed = constant("<i8 1, i8 poison, i8 3, i8 4>", 4, 8).unwrap();
        let pairs = bitcast(&mixed, "<4 x i8>", "<2 x i16>").unwrap();
        let pairs = lanes(&pairs, 2, 16).unwrap();
        assert_eq!(pairs[0].defined, "false");
        assert_eq!(pairs[1].defined, "true");
        assert!(bitcast(&mixed, "<4 x i8>", "i16").is_err());
    }
    #[test]
    fn types_and_constants() {
        assert_eq!(vector_type("<16 x i8>"), Some((16, 8)));
        assert_eq!(vector_type("<2 x ptr>"), None);
        assert_eq!(vector_type("<{ i8 }>"), None);
        let splat = constant("splat (i8 -1)", 3, 8).unwrap();
        assert!(
            lanes(&splat, 3, 8)
                .unwrap()
                .iter()
                .all(|l| l.expr == bv(255, 8))
        );
        assert!(constant("<i8 1, i8 2>", 3, 8).is_err());
        assert_eq!(
            split_vector("nuw <2 x i64> %x"),
            Some((vec!["nuw"], "<2 x i64>", "%x"))
        );
    }
}
