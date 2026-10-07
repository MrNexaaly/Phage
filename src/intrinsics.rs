//! Integer LLVM intrinsics with exact LangRef semantics: min/max, abs,
//! byte/bit reversal, bit counts, funnel shifts, saturating arithmetic and
//! three-way comparison. Poison flags are honored; other intrinsics stay
//! unknown in `execute.rs`.

use crate::{
    engine::{Engine, State},
    value::{Value, and, bv},
};

impl Engine {
    /// Returns `None` when `callee` is not an intrinsic modeled here.
    pub fn intrinsic(
        &mut self,
        state: &State,
        callee: &str,
        args: &[&str],
    ) -> Result<Option<Value>, String> {
        let Some(name) = callee.strip_prefix("llvm.") else {
            return Ok(None);
        };
        let family = name.split('.').next().unwrap_or("");
        let arity = match family {
            "umin" | "umax" | "smin" | "smax" | "abs" | "ctlz" | "cttz" | "uadd" | "usub"
            | "sadd" | "ssub" | "ucmp" | "scmp" | "expect" => 2,
            "bswap" | "bitreverse" | "ctpop" => 1,
            "fshl" | "fshr" => 3,
            _ => return Ok(None),
        };
        // `uadd.with.overflow` and friends are handled by the caller.
        if matches!(family, "uadd" | "usub" | "sadd" | "ssub") && !name.contains(".sat.") {
            return Ok(None);
        }
        if args.len() != arity {
            return Err(format!("{callee} expects {arity} operands"));
        }
        let a = self.typed_operand(state, args[0])?;
        let width = a.width()?;
        let top = width - 1;
        let bit = |i: u32| format!("((_ extract {i} {i}) {})", a.expr);
        let value = match family {
            "umin" | "umax" | "smin" | "smax" => {
                let b = self.typed_operand(state, args[1])?;
                let cmp = match family {
                    "umin" => "bvult",
                    "umax" => "bvugt",
                    "smin" => "bvslt",
                    _ => "bvsgt",
                };
                Value::bits(
                    format!("(ite ({cmp} {} {}) {} {})", a.expr, b.expr, a.expr, b.expr),
                    width,
                    and(&[a.defined.clone(), b.defined.clone()]),
                )
            }
            "abs" => {
                let poison_min = self.flag(state, args[1])?;
                let min = bv(1 << top, width);
                let mut defined = vec![a.defined.clone()];
                if poison_min {
                    defined.push(format!("(not (= {} {min}))", a.expr));
                }
                Value::bits(
                    format!(
                        "(ite (bvslt {} {}) (bvneg {}) {})",
                        a.expr,
                        bv(0, width),
                        a.expr,
                        a.expr
                    ),
                    width,
                    and(&defined),
                )
            }
            "bswap" => {
                if width % 16 != 0 {
                    return Err(format!("{callee} needs an even number of bytes"));
                }
                // The first concat operand is most significant: input byte 0.
                let bytes: Vec<_> = (0..width / 8)
                    .map(|i| format!("((_ extract {} {}) {})", i * 8 + 7, i * 8, a.expr))
                    .collect();
                Value::bits(
                    format!("(concat {})", bytes.join(" ")),
                    width,
                    a.defined.clone(),
                )
            }
            "bitreverse" => {
                let expr = if width == 1 {
                    a.expr.clone()
                } else {
                    format!(
                        "(concat {})",
                        (0..width).map(bit).collect::<Vec<_>>().join(" ")
                    )
                };
                Value::bits(expr, width, a.defined.clone())
            }
            "ctpop" => {
                // A balanced adder tree whose sums grow one bit per level
                // bit-blasts to O(width) gates, unlike width-wide additions.
                let mut terms: Vec<(String, u32)> = (0..width).map(|i| (bit(i), 1)).collect();
                while terms.len() > 1 {
                    terms = terms
                        .chunks(2)
                        .map(|pair| match pair {
                            [(x, wx), (y, wy)] => {
                                let w = wx.max(wy) + 1;
                                (
                                    format!(
                                        "(bvadd ((_ zero_extend {}) {x}) ((_ zero_extend {}) {y}))",
                                        w - wx,
                                        w - wy
                                    ),
                                    w,
                                )
                            }
                            [single] => single.clone(),
                            _ => unreachable!("chunks of two"),
                        })
                        .collect();
                }
                let (sum, bits) = terms.pop().ok_or("ctpop of zero width")?;
                let expr = if bits < width {
                    format!("((_ zero_extend {}) {sum})", width - bits)
                } else {
                    sum
                };
                Value::bits(expr, width, a.defined.clone())
            }
            "ctlz" | "cttz" => {
                let poison_zero = self.flag(state, args[1])?;
                // Innermost case: no set bit, the count is the full width.
                let mut expr = bv(u128::from(width), width);
                for count in (0..width).rev() {
                    let index = if family == "ctlz" { top - count } else { count };
                    expr = format!(
                        "(ite (= {} {}) {} {expr})",
                        bit(index),
                        bv(1, 1),
                        bv(u128::from(count), width)
                    );
                }
                let mut defined = vec![a.defined.clone()];
                if poison_zero {
                    defined.push(format!("(not (= {} {}))", a.expr, bv(0, width)));
                }
                Value::bits(expr, width, and(&defined))
            }
            "fshl" | "fshr" => {
                let b = self.typed_operand(state, args[1])?;
                let c = self.typed_operand(state, args[2])?;
                // Shift the 2N-bit concatenation by c mod N; exact for any N.
                let amount = format!(
                    "((_ zero_extend {width}) (bvurem {} {}))",
                    c.expr,
                    bv(u128::from(width), width)
                );
                let joined = format!("(concat {} {})", a.expr, b.expr);
                let expr = if family == "fshl" {
                    format!(
                        "((_ extract {} {width}) (bvshl {joined} {amount}))",
                        2 * width - 1
                    )
                } else {
                    format!("((_ extract {top} 0) (bvlshr {joined} {amount}))")
                };
                Value::bits(
                    expr,
                    width,
                    and(&[a.defined.clone(), b.defined.clone(), c.defined.clone()]),
                )
            }
            "uadd" | "usub" | "sadd" | "ssub" => {
                let b = self.typed_operand(state, args[1])?;
                let (x, y) = (&a.expr, &b.expr);
                let expr = match family {
                    "uadd" => format!(
                        "(ite (bvult (bvadd {x} {y}) {x}) {} (bvadd {x} {y}))",
                        bv(u128::MAX >> (128 - width), width)
                    ),
                    "usub" => format!("(ite (bvult {x} {y}) {} (bvsub {x} {y}))", bv(0, width)),
                    _ => {
                        // Exact N+1-bit result; clamp by the sign of `a`, the
                        // only possible overflow direction for add and sub.
                        let op = if family == "sadd" { "bvadd" } else { "bvsub" };
                        let wide =
                            format!("({op} ((_ sign_extend 1) {x}) ((_ sign_extend 1) {y}))");
                        let overflow = format!(
                            "(not (= ((_ extract {width} {width}) {wide}) ((_ extract {top} {top}) {wide})))"
                        );
                        let max = bv((1u128 << top) - 1, width);
                        let min = bv(1u128 << top, width);
                        format!(
                            "(ite {overflow} (ite (bvslt {x} {}) {min} {max}) ((_ extract {top} 0) {wide}))",
                            bv(0, width)
                        )
                    }
                };
                Value::bits(expr, width, and(&[a.defined.clone(), b.defined.clone()]))
            }
            "ucmp" | "scmp" => {
                let b = self.typed_operand(state, args[1])?;
                // The first overload suffix is the result type: llvm.scmp.i8.i32.
                let result = name
                    .split('.')
                    .nth(1)
                    .and_then(|t| t.strip_prefix('i'))
                    .and_then(|t| t.parse::<u32>().ok())
                    .filter(|w| (2..=128).contains(w))
                    .ok_or_else(|| format!("{callee} result type is unsupported"))?;
                let less = if family == "ucmp" { "bvult" } else { "bvslt" };
                Value::bits(
                    format!(
                        "(ite ({less} {} {}) {} (ite (= {} {}) {} {}))",
                        a.expr,
                        b.expr,
                        bv(u128::MAX >> (128 - result), result),
                        a.expr,
                        b.expr,
                        bv(0, result),
                        bv(1, result)
                    ),
                    result,
                    and(&[a.defined.clone(), b.defined.clone()]),
                )
            }
            // A branch-weight hint: the value is returned unchanged.
            "expect" => a,
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    /// Immediate `i1` flag operands must be literal constants.
    fn flag(&mut self, state: &State, text: &str) -> Result<bool, String> {
        let value = self.typed_operand(state, text)?;
        if value.expr == bv(1, 1) {
            Ok(true)
        } else if value.expr == bv(0, 1) {
            Ok(false)
        } else {
            Err("intrinsic flag operand must be constant".into())
        }
    }
}

#[cfg(test)]
mod tests;
