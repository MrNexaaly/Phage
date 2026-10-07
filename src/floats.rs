//! IEEE-754 `float` and `double`. A value of kind `Float(e, s)` holds its
//! IEEE bits as a bit-vector, so the operations LangRef defines on bits —
//! `bitcast`, load, store, select, phi, `fneg`, `fabs`, `copysign` — are
//! exact, NaN payloads included. Arithmetic, conversions and comparisons
//! read the bits as an SMT-LIB FloatingPoint term (`to_fp`), round to
//! nearest even (LLVM's default environment), and a computed float gets
//! fresh bits `b` with `to_fp(b) = result`: every non-NaN result has exactly
//! one encoding (±0 stay distinct); a NaN result is quiet with any payload,
//! a superset of LangRef's choices (a quieted input payload or the
//! preferred NaN), so only a property that depends on the payload of a
//! computed NaN could see a counterexample LLVM cannot produce. Conversions to
//! integers that do not fit are poison; the `.sat` intrinsics saturate.
//! The solver switches to logic ALL the first time a program uses floating
//! point. `frem` (fmod), fast-math flags other than `nnan`/`ninf`, other
//! formats, vectors and library math stay unknown.

use crate::{
    engine::{Engine, State},
    ir::{split, typed},
    value::{Kind, Value, and, bv, not},
};

/// Exponent and significand bits of `float` and `double`.
pub fn format(ty: &str) -> Option<(u32, u32)> {
    match ty.trim() {
        "float" => Some((8, 24)),
        "double" => Some((11, 53)),
        _ => None,
    }
}

fn width((e, s): (u32, u32)) -> u32 {
    e + s
}

/// The FloatingPoint reading of a float value's bits.
fn fp(value: &Value, (e, s): (u32, u32)) -> String {
    format!("((_ to_fp {e} {s}) {})", value.expr)
}

fn bits_of(x: f64, format: (u32, u32)) -> u128 {
    match format {
        (8, 24) => u128::from((x as f32).to_bits()),
        _ => u128::from(x.to_bits()),
    }
}

/// An exactly representable value as an FP literal.
fn exact(x: f64, format: (u32, u32)) -> String {
    format!(
        "((_ to_fp {} {}) {})",
        format.0,
        format.1,
        bv(bits_of(x, format), width(format))
    )
}

/// 2^k as an FP literal, or +infinity past the largest finite value.
fn power(k: u32, format: (u32, u32)) -> String {
    let finite = match format {
        (8, 24) => k < 128,
        _ => k < 1024,
    };
    if finite {
        exact(2f64.powi(k as i32), format)
    } else {
        format!("(_ +oo {} {})", format.0, format.1)
    }
}

/// An LLVM floating-point constant's bits: decimal (`5.000000e-01`) or the
/// 64-bit hexadecimal double form (`0x3FE0000000000000`), which LLVM also
/// prints for `float` (always exactly representable as `float`).
pub fn constant(text: &str, format: (u32, u32)) -> Result<String, String> {
    let double = if let Some(hex) = text.strip_prefix("0x") {
        if hex.len() != 16 {
            return Err(format!("unsupported floating-point constant {text}"));
        }
        f64::from_bits(u64::from_str_radix(hex, 16).map_err(|e| e.to_string())?)
    } else {
        text.parse::<f64>()
            .map_err(|_| format!("unsupported floating-point constant {text}"))?
    };
    if format == (8, 24) {
        let single = double as f32;
        if double.is_nan() {
            // LLVM keeps a NaN's high payload bits when narrowing the hex form.
            let bits = double.to_bits();
            let payload = ((bits >> 29) & 0x7f_ffff) as u32;
            let sign = ((bits >> 63) as u32) << 31;
            return Ok(bv(u128::from(sign | 0x7f80_0000 | payload), 32));
        }
        if f64::from(single) != double {
            return Err(format!("inexact float constant {text}"));
        }
    }
    Ok(bv(bits_of(double, format), width(format)))
}

/// Two floating-point operands, their format, `nnan`/`ninf`, and the word
/// between the flags and the type (an `fcmp` predicate).
struct Pair {
    a: Value,
    b: Value,
    format: (u32, u32),
    nnan: bool,
    ninf: bool,
    lead: String,
}

fn float(expr: String, format: (u32, u32), defined: String) -> Value {
    Value {
        expr,
        kind: Kind::Float(format.0, format.1),
        defined,
    }
}

impl Engine {
    /// Switches the solver to a logic with floating point, once.
    pub fn floats_on(&mut self) -> Result<(), String> {
        self.solver.enable_floats()
    }

    /// Fresh bits whose FloatingPoint reading is `result` (any NaN payload
    /// when `result` is NaN).
    fn computed(
        &mut self,
        state: &mut State,
        result: String,
        format: (u32, u32),
        defined: String,
    ) -> Result<Value, String> {
        let bits = self.symbol(&format!("(_ BitVec {})", width(format)))?;
        state.constraints.push(format!(
            "(= ((_ to_fp {} {}) {bits}) {result})",
            format.0, format.1
        ));
        // LangRef: a NaN produced by arithmetic is quiet.
        let quiet = format.1 - 2;
        state.constraints.push(format!(
            "(or (not (fp.isNaN {result})) (= ((_ extract {quiet} {quiet}) {bits}) {}))",
            bv(1, 1)
        ));
        Ok(float(bits, format, defined))
    }

    fn float_operand(&mut self, state: &State, text: &str) -> Result<(Value, (u32, u32)), String> {
        let (ty, _) = typed(text)?;
        let format = format(ty).ok_or_else(|| format!("unsupported floating-point type {ty}"))?;
        let value = self.typed_operand(state, text)?;
        match value.kind {
            Kind::Float(e, s) if (e, s) == format => Ok((value, format)),
            _ => Err("floating-point operand expected".into()),
        }
    }

    /// Fast-math flags before the type: `nnan` and `ninf` make NaN or
    /// infinite operands and results poison; the others license rewrites
    /// whose results are not modeled, so they are unknown.
    fn fast_math(words: &[&str]) -> Result<(bool, bool, usize), String> {
        let (mut nnan, mut ninf, mut used) = (false, false, 0);
        for word in words {
            match *word {
                "nnan" => nnan = true,
                "ninf" => ninf = true,
                "nsz" | "arcp" | "contract" | "afn" | "reassoc" | "fast" => {
                    return Err(format!("unsupported fast-math flag {word}"));
                }
                _ => break,
            }
            used += 1;
        }
        Ok((nnan, ninf, used))
    }

    fn flag_conditions(nnan: bool, ninf: bool, terms: &[&str]) -> Vec<String> {
        let mut valid = Vec::new();
        for term in terms {
            if nnan {
                valid.push(not(&format!("(fp.isNaN {term})")));
            }
            if ninf {
                valid.push(not(&format!("(fp.isInfinite {term})")));
            }
        }
        valid
    }

    /// The two typed operands of `OP [flags] [LEAD] TY A, B`.
    fn float_pair(&mut self, state: &State, rest: &str, lead: bool) -> Result<Pair, String> {
        let args = split(rest);
        let words: Vec<&str> = args
            .first()
            .ok_or("operands missing")?
            .split_whitespace()
            .collect();
        let (nnan, ninf, used) = Self::fast_math(&words)?;
        let skip = usize::from(lead);
        if words.len() < used + skip + 2 {
            return Err("floating-point operands missing".into());
        }
        let ty = words[used + skip];
        let (a, format) = self.float_operand(state, &words[used + skip..].join(" "))?;
        let second = args.get(1).ok_or("operand missing")?;
        let (b, _) = self.float_operand(state, &format!("{ty} {second}"))?;
        Ok(Pair {
            a,
            b,
            format,
            nnan,
            ninf,
            lead: if lead {
                words[used].to_owned()
            } else {
                String::new()
            },
        })
    }

    /// Floating-point instructions; `None` for other opcodes.
    pub fn float_instruction(
        &mut self,
        state: &mut State,
        opcode: &str,
        rest: &str,
    ) -> Result<Option<Value>, String> {
        let value = match opcode {
            "frem" => return Err("frem (fmod) is unsupported".into()),
            "fadd" | "fsub" | "fmul" | "fdiv" => {
                let Pair {
                    a,
                    b,
                    format,
                    nnan,
                    ninf,
                    ..
                } = self.float_pair(state, rest, false)?;
                let (x, y) = (fp(&a, format), fp(&b, format));
                let result = format!("(fp.{} RNE {x} {y})", &opcode[1..]);
                let mut valid = vec![a.defined.clone(), b.defined.clone()];
                valid.extend(Self::flag_conditions(nnan, ninf, &[&x, &y, &result]));
                self.computed(state, result, format, and(&valid))?
            }
            "fneg" => {
                // LangRef: fneg flips the sign bit only.
                let words: Vec<&str> = rest.split_whitespace().collect();
                let (nnan, ninf, used) = Self::fast_math(&words)?;
                let (a, format) = self.float_operand(state, &words[used..].join(" "))?;
                let sign = bv(1u128 << (width(format) - 1), width(format));
                let mut valid = vec![a.defined.clone()];
                valid.extend(Self::flag_conditions(nnan, ninf, &[&fp(&a, format)]));
                float(format!("(bvxor {} {sign})", a.expr), format, and(&valid))
            }
            "fcmp" => {
                let Pair {
                    a,
                    b,
                    format,
                    nnan,
                    ninf,
                    lead,
                } = self.float_pair(state, rest, true)?;
                let (x, y) = (fp(&a, format), fp(&b, format));
                let unordered = format!("(or (fp.isNaN {x}) (fp.isNaN {y}))");
                let or_unordered = |c: String| format!("(or {unordered} {c})");
                let condition = match lead.as_str() {
                    "false" => "false".to_owned(),
                    "true" => "true".to_owned(),
                    "oeq" => format!("(fp.eq {x} {y})"),
                    "ogt" => format!("(fp.gt {x} {y})"),
                    "oge" => format!("(fp.geq {x} {y})"),
                    "olt" => format!("(fp.lt {x} {y})"),
                    "ole" => format!("(fp.leq {x} {y})"),
                    "one" => format!("(and (not {unordered}) (not (fp.eq {x} {y})))"),
                    "ord" => not(&unordered),
                    "ueq" => or_unordered(format!("(fp.eq {x} {y})")),
                    "ugt" => or_unordered(format!("(fp.gt {x} {y})")),
                    "uge" => or_unordered(format!("(fp.geq {x} {y})")),
                    "ult" => or_unordered(format!("(fp.lt {x} {y})")),
                    "ule" => or_unordered(format!("(fp.leq {x} {y})")),
                    "une" => format!("(not (fp.eq {x} {y}))"),
                    "uno" => unordered.clone(),
                    other => return Err(format!("unsupported fcmp predicate {other}")),
                };
                let mut valid = vec![a.defined.clone(), b.defined.clone()];
                valid.extend(Self::flag_conditions(nnan, ninf, &[&x, &y]));
                Value::bits(
                    format!("(ite {condition} {} {})", bv(1, 1), bv(0, 1)),
                    1,
                    and(&valid),
                )
            }
            "uitofp" | "sitofp" | "fptoui" | "fptosi" | "fpext" | "fptrunc" => {
                let (source, target) =
                    rest.split_once(" to ").ok_or("conversion target missing")?;
                let words: Vec<&str> = source.split_whitespace().collect();
                let nneg = words.first() == Some(&"nneg");
                let source = words[usize::from(nneg)..].join(" ");
                let value = self.float_conversion(state, opcode, &source, target.trim())?;
                if nneg {
                    // `uitofp nneg`: a negative operand makes the result poison.
                    let x = self.typed_operand(state, &source)?;
                    let top = x.width()? - 1;
                    let positive = format!("(= ((_ extract {top} {top}) {}) {})", x.expr, bv(0, 1));
                    crate::partial::restrict(value, &positive)
                } else {
                    value
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn float_conversion(
        &mut self,
        state: &mut State,
        opcode: &str,
        source: &str,
        target: &str,
    ) -> Result<Value, String> {
        match opcode {
            "uitofp" | "sitofp" => {
                let format = format(target).ok_or("unsupported conversion target")?;
                self.floats_on()?;
                let x = self.typed_operand(state, source)?;
                x.width()?;
                // `to_fp` with a rounding mode reads the vector as signed.
                let to = if opcode == "uitofp" {
                    "to_fp_unsigned"
                } else {
                    "to_fp"
                };
                let result = format!("((_ {to} {} {}) RNE {})", format.0, format.1, x.expr);
                self.computed(state, result, format, x.defined.clone())
            }
            "fptoui" | "fptosi" => {
                let bits: u32 = target
                    .strip_prefix('i')
                    .and_then(|w| w.parse().ok())
                    .ok_or("unsupported conversion target")?;
                let (f, format) = self.float_operand(state, source)?;
                let x = fp(&f, format);
                // LangRef: poison unless the truncated value fits.
                let fits = if opcode == "fptoui" {
                    format!(
                        "(and (fp.gt {x} {}) (fp.lt {x} {}))",
                        exact(-1.0, format),
                        power(bits, format)
                    )
                } else {
                    format!(
                        "(and (fp.geq (fp.roundToIntegral RTZ {x}) (fp.neg {})) (fp.lt {x} {}))",
                        power(bits - 1, format),
                        power(bits - 1, format)
                    )
                };
                let to = if opcode == "fptoui" {
                    "fp.to_ubv"
                } else {
                    "fp.to_sbv"
                };
                Ok(Value::bits(
                    format!("((_ {to} {bits}) RTZ {x})"),
                    bits,
                    and(&[f.defined.clone(), fits]),
                ))
            }
            _ => {
                let to = format(target).ok_or("unsupported conversion target")?;
                let (f, from) = self.float_operand(state, source)?;
                let result = format!("((_ to_fp {} {}) RNE {})", to.0, to.1, fp(&f, from));
                self.computed(state, result, to, f.defined)
            }
        }
    }

    /// Bits and floats share one encoding: `bitcast`, loads and stores only
    /// relabel the kind.
    pub fn float_bits(value: &Value) -> Value {
        match value.kind {
            Kind::Float(e, s) => Value::bits(value.expr.clone(), e + s, value.defined.clone()),
            _ => value.clone(),
        }
    }

    pub fn bits_float(&mut self, value: &Value, format: (u32, u32)) -> Result<Value, String> {
        self.floats_on()?;
        if value.width()? != width(format) {
            return Err("bitcast width mismatch".into());
        }
        Ok(float(value.expr.clone(), format, value.defined.clone()))
    }

    /// Floating-point intrinsics with the call's fast-math flags (`nnan`
    /// and `ninf` poison NaN or infinite operands and results; others are
    /// unknown); `None` for other callees.
    pub fn float_intrinsic(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
        flags: &[&str],
    ) -> Result<Option<Value>, String> {
        let Some(value) = self.float_intrinsic_value(state, callee, args)? else {
            if !flags.is_empty() {
                return Err(format!(
                    "fast-math flags on a call to {callee} are unsupported"
                ));
            }
            return Ok(None);
        };
        let (nnan, ninf, used) = Self::fast_math(flags)?;
        if used < flags.len() {
            return Err("unsupported fast-math flag on a call".into());
        }
        if !nnan && !ninf {
            return Ok(Some(value));
        }
        let mut terms = Vec::new();
        for arg in args.iter().filter(|a| !a.is_empty()) {
            let (operand, f) = self.float_operand(state, arg)?;
            terms.push(fp(&operand, f));
        }
        if let Kind::Float(e, s) = value.kind {
            terms.push(fp(&value, (e, s)));
        }
        let refs: Vec<&str> = terms.iter().map(String::as_str).collect();
        let valid = and(&Self::flag_conditions(nnan, ninf, &refs));
        Ok(Some(crate::partial::restrict(value, &valid)))
    }

    fn float_intrinsic_value(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
    ) -> Result<Option<Value>, String> {
        let Some(name) = callee.strip_prefix("llvm.") else {
            return Ok(None);
        };
        let family = name.split('.').next().unwrap_or("");
        let args: Vec<&str> = args.iter().copied().filter(|a| !a.is_empty()).collect();
        if name.starts_with("is.fpclass.") {
            return self.fpclass(state, &args).map(Some);
        }
        let arity = match family {
            "fabs" | "sqrt" | "floor" | "ceil" | "trunc" | "round" | "roundeven" | "rint"
            | "nearbyint" => 1,
            "minnum" | "maxnum" | "copysign" => 2,
            "fma" => 3,
            "fptoui" | "fptosi" if name.contains(".sat.") => 1,
            _ => return Ok(None),
        };
        if args.len() != arity {
            return Err(format!("{callee} expects {arity} operands"));
        }
        let mut operands = Vec::new();
        let mut format = (0, 0);
        for arg in &args {
            let (value, f) = self.float_operand(state, arg)?;
            format = f;
            operands.push(value);
        }
        let defined = and(&operands
            .iter()
            .map(|v| v.defined.clone())
            .collect::<Vec<_>>());
        let w = width(format);
        let sign = 1u128 << (w - 1);
        let x = fp(&operands[0], format);
        // Sign operations act on bits (LangRef), so they are exact.
        match family {
            "fabs" => {
                let expr = format!("(bvand {} {})", operands[0].expr, bv(sign - 1, w));
                return Ok(Some(float(expr, format, defined)));
            }
            "copysign" => {
                let expr = format!(
                    "(bvor (bvand {} {}) (bvand {} {}))",
                    operands[0].expr,
                    bv(sign - 1, w),
                    operands[1].expr,
                    bv(sign, w)
                );
                return Ok(Some(float(expr, format, defined)));
            }
            "fptoui" | "fptosi" => {
                let bits: u32 = name
                    .split('.')
                    .nth(2)
                    .and_then(|t| t.strip_prefix('i'))
                    .and_then(|w| w.parse().ok())
                    .ok_or("saturating conversion width missing")?;
                let expr = if family == "fptoui" {
                    format!(
                        "(ite (fp.isNaN {x}) {zero} (ite (fp.geq {x} {top}) {max} (ite (fp.leq {x} {neg}) {zero} ((_ fp.to_ubv {bits}) RTZ {x}))))",
                        zero = bv(0, bits),
                        top = power(bits, format),
                        max = bv(crate::knownbits::mask(bits), bits),
                        neg = exact(-1.0, format),
                    )
                } else {
                    let half = 1u128 << (bits - 1);
                    format!(
                        "(ite (fp.isNaN {x}) {zero} (ite (fp.geq {x} {top}) {max} (ite (fp.leq {x} (fp.neg {top})) {min} ((_ fp.to_sbv {bits}) RTZ {x}))))",
                        zero = bv(0, bits),
                        top = power(bits - 1, format),
                        max = bv(half - 1, bits),
                        min = bv(half, bits),
                    )
                };
                return Ok(Some(Value::bits(expr, bits, defined)));
            }
            _ => {}
        }
        let terms: Vec<String> = operands.iter().map(|v| fp(v, format)).collect();
        let result = match family {
            "sqrt" => format!("(fp.sqrt RNE {x})"),
            "floor" => format!("(fp.roundToIntegral RTN {x})"),
            "ceil" => format!("(fp.roundToIntegral RTP {x})"),
            "trunc" => format!("(fp.roundToIntegral RTZ {x})"),
            "round" => format!("(fp.roundToIntegral RNA {x})"),
            // Opposite zeros: either, chosen afresh by each operation.
            "minnum" | "maxnum" => {
                let (x, y) = (&terms[0], &terms[1]);
                let either = self.symbol("Bool")?;
                format!(
                    "(ite (and (fp.isZero {x}) (fp.isZero {y})) (ite {either} {x} {y}) (fp.{} {x} {y}))",
                    &family[..3]
                )
            }
            "fma" => format!("(fp.fma RNE {} {} {})", terms[0], terms[1], terms[2]),
            _ => format!("(fp.roundToIntegral RNE {x})"),
        };
        Ok(Some(self.computed(state, result, format, defined)?))
    }
}

impl Engine {
    /// `llvm.is.fpclass(x, mask)`: LangRef's ten classes read from the bits
    /// (bit 0 signaling NaN, 1 quiet NaN, 2..=5 negative infinity, normal,
    /// subnormal, zero, 6..=9 positive zero, subnormal, normal, infinity).
    fn fpclass(&mut self, state: &State, args: &[&str]) -> Result<Value, String> {
        let [value, mask] = args else {
            return Err("llvm.is.fpclass expects 2 operands".into());
        };
        let (x, format) = self.float_operand(state, value)?;
        let mask: u32 = mask
            .split_whitespace()
            .last()
            .and_then(|m| m.parse().ok())
            .ok_or("llvm.is.fpclass needs a constant mask")?;
        let (e, s) = format;
        let w = e + s;
        let field = |high: u32, low: u32| format!("((_ extract {high} {low}) {})", x.expr);
        let exponent = field(w - 2, s - 1);
        let fraction = field(s - 2, 0);
        let ones = format!("(= {exponent} {})", bv(crate::knownbits::mask(e), e));
        let zeros = format!("(= {exponent} {})", bv(0, e));
        let empty = format!("(= {fraction} {})", bv(0, s - 1));
        let quiet = format!("(= {} {})", field(s - 2, s - 2), bv(1, 1));
        let negative = format!("(= {} {})", field(w - 1, w - 1), bv(1, 1));
        let nan = format!("(and {ones} (not {empty}))");
        let classes = [
            format!("(and {nan} (not {quiet}))"),
            format!("(and {nan} {quiet})"),
            format!("(and {negative} {ones} {empty})"),
            format!("(and {negative} (not {ones}) (not {zeros}))"),
            format!("(and {negative} {zeros} (not {empty}))"),
            format!("(and {negative} {zeros} {empty})"),
            format!("(and (not {negative}) {zeros} {empty})"),
            format!("(and (not {negative}) {zeros} (not {empty}))"),
            format!("(and (not {negative}) (not {ones}) (not {zeros}))"),
            format!("(and (not {negative}) {ones} {empty})"),
        ];
        let chosen: Vec<String> = (0..10)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| classes[bit].clone())
            .collect();
        let test = crate::value::or(&chosen);
        let test = if chosen.is_empty() {
            "false".to_owned()
        } else {
            test
        };
        Ok(Value::bits(
            format!("(ite {test} {} {})", bv(1, 1), bv(0, 1)),
            1,
            x.defined,
        ))
    }
}

#[cfg(test)]
mod tests;
