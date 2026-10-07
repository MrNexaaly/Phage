//! Built-MIR operands and rvalues: integers, aggregates, enum discriminants,
//! casts and references. Rust checked arithmetic retains its overflow flag.

use super::{
    Data, Node, State, Verifier, integer,
    parse::{closing, split},
};
use crate::value::{self, Value, bv};

impl Verifier {
    pub fn operand(&mut self, state: &mut State, text: &str) -> Result<Node, String> {
        let text = text.trim();
        if let Some(place) = text.strip_prefix("copy ") {
            return self.read_place(state, place, false);
        }
        if let Some(place) = text.strip_prefix("move ") {
            return self.read_place(state, place, true);
        }
        if let Some(constant) = text.strip_prefix("const ") {
            return self.constant(state, constant);
        }
        Err(format!("unsupported MIR operand {text}"))
    }
    fn constant(&mut self, state: &State, text: &str) -> Result<Node, String> {
        let text = if let Some(ty) = self
            .program
            .registry
            .constant_types
            .get(&(state.frame()?.function.clone(), text.into()))
            .or_else(|| {
                self.program.registry.constant_types.get(&(
                    self.program
                        .functions
                        .get(&state.frame().ok()?.function)
                        .map(|f| f.name.clone())
                        .unwrap_or_default(),
                    text.into(),
                ))
            }) {
            format!("{}_{}", state.ty(text), state.ty(ty))
        } else {
            state.ty(text)
        };
        if matches!(text.as_str(), "true" | "false") {
            return Ok(Node::scalar("bool", value::constant(&text, 1)?));
        }
        if let Some(boolean) = text.strip_suffix("_bool") {
            return Ok(Node::scalar("bool", value::constant(boolean, 1)?));
        }
        if text.starts_with('"') && text.ends_with('"') {
            let bytes = decode(&text[1..text.len() - 1])?;
            return Ok(Node {
                ty: "&str".into(),
                kind: Data::Str {
                    bytes: bytes
                        .into_iter()
                        .map(|b| {
                            Node::scalar("u8", Value::bits(bv(u128::from(b), 8), 8, "true".into()))
                        })
                        .collect(),
                    owner: None,
                },
            });
        }
        if text == "()" {
            return Ok(Node {
                ty: "()".into(),
                kind: Data::Tuple(Vec::new()),
            });
        }
        if let Some((ty, name)) = text.rsplit_once("::")
            && let Some((width, signed)) = integer(ty)
        {
            let bits = match name {
                "MAX" => {
                    if signed {
                        (1u128 << (width - 1)) - 1
                    } else if width == 128 {
                        u128::MAX
                    } else {
                        (1u128 << width) - 1
                    }
                }
                "MIN" => {
                    if signed {
                        1u128 << (width - 1)
                    } else {
                        0
                    }
                }
                _ => return Err(format!("unsupported MIR constant {text}")),
            };
            return Ok(Node::scalar(
                ty,
                Value::bits(bv(bits, width), width, "true".into()),
            ));
        }
        let (number, ty) = text
            .rsplit_once('_')
            .filter(|(_, t)| integer(t).is_some())
            .ok_or_else(|| format!("unsupported MIR constant {text}"))?;
        let (width, _) = integer(ty).ok_or("MIR constant type unsupported")?;
        let number = number.replace('_', "");
        let number = if let Some(hex) = number.strip_prefix("0x") {
            u128::from_str_radix(hex, 16)
                .map_err(|_| "MIR hexadecimal constant invalid")?
                .to_string()
        } else {
            number
        };
        Ok(Node::scalar(ty, value::constant(&number, width)?))
    }
    pub fn expression(
        &mut self,
        state: &mut State,
        text: &str,
        expected: &str,
    ) -> Result<Node, String> {
        let text = text.trim();
        if let Some((source, target)) = text.rsplit_once(" as ")
            && target.contains(" (")
        {
            let (ty, cast) = target.rsplit_once(" (").ok_or("MIR cast kind missing")?;
            if cast != "IntToInt)" {
                return Err(format!("unsupported MIR cast {cast}"));
            }
            let source = self.operand(state, source)?;
            return self.cast(source, &state.ty(ty));
        }
        if text.starts_with("copy ") || text.starts_with("move ") || text.starts_with("const ") {
            return self.operand(state, text);
        }
        if let Some(place) = text
            .strip_prefix("&mut ")
            .or_else(|| text.strip_prefix('&'))
        {
            if let Some(inner) = super::parse::parentheses(place).strip_prefix('*') {
                let value = self.read_place(state, inner, false)?;
                if matches!(value.kind, Data::Str { .. }) {
                    return Ok(Node {
                        ty: expected.into(),
                        kind: value.kind,
                    });
                }
            }
            let address = self.address(state, place)?;
            self.read_address(state, &address)?;
            return Ok(Node {
                ty: expected.into(),
                kind: Data::Ref(address),
            });
        }
        if let Some(place) = text
            .strip_prefix("discriminant(")
            .and_then(|s| s.strip_suffix(')'))
        {
            let node = self.read_place(state, place, false)?;
            let Data::Enum { tag, .. } = node.kind else {
                return Err("MIR discriminant requires enum".into());
            };
            return Ok(Node::scalar(
                expected,
                value::constant(
                    &tag.to_string(),
                    integer(expected).ok_or("discriminant type unsupported")?.0,
                )?,
            ));
        }
        if let Some(place) = text.strip_prefix("Len(").and_then(|s| s.strip_suffix(')')) {
            let node = self.read_place(state, place, false)?;
            let Data::Array(values) = node.kind else {
                return Err("unsupported MIR Len of non-array".into());
            };
            return Ok(Node::scalar(
                "usize",
                value::constant(&values.len().to_string(), 64)?,
            ));
        }
        if text.starts_with('[') && closing(text, 0) == Some(text.len() - 1) {
            let inner = &text[1..text.len() - 1];
            let values = if let Some((value, count)) = inner.split_once("; ") {
                let count = state
                    .ty(count)
                    .parse::<usize>()
                    .map_err(|_| "MIR repeat count unsupported")?;
                if count > 4096 {
                    return Err("MIR array exceeds prototype limit".into());
                }
                vec![self.operand(state, value)?; count]
            } else {
                split(inner)
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| self.operand(state, s))
                    .collect::<Result<Vec<_>, _>>()?
            };
            return Ok(Node {
                ty: expected.into(),
                kind: Data::Array(values),
            });
        }
        if text.starts_with('(') && closing(text, 0) == Some(text.len() - 1) {
            let values = split(&text[1..text.len() - 1])
                .into_iter()
                .filter(|s| !s.is_empty())
                .map(|s| self.operand(state, s))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Node {
                ty: expected.into(),
                kind: Data::Tuple(values),
            });
        }
        if let Some((name, fields)) = text.split_once(" { ").filter(|_| text.ends_with(" }")) {
            let definition = self
                .program
                .registry
                .definition(&state.ty(name))
                .cloned()
                .ok_or_else(|| format!("unsupported MIR struct {name}"))?;
            if !definition.supported || !definition.variants.is_empty() {
                return Err("unsupported MIR struct declaration".into());
            }
            let mut fields_by_name = std::collections::BTreeMap::new();
            for field in split(fields.trim_end_matches(" }")) {
                let (name, value) = field.split_once(": ").ok_or("MIR struct field missing")?;
                fields_by_name.insert(name, self.operand(state, value)?);
            }
            let values = definition
                .fields
                .iter()
                .map(|name| {
                    fields_by_name
                        .remove(name.as_str())
                        .ok_or("MIR struct field absent".into())
                })
                .collect::<Result<Vec<_>, String>>()?;
            if !fields_by_name.is_empty() {
                return Err("MIR extra struct field".into());
            }
            return Ok(Node {
                ty: expected.into(),
                kind: Data::Struct(values),
            });
        }
        if let Some(open) = text.find('(').filter(|_| text.ends_with(')')) {
            let name = &text[..open];
            let args = split(&text[open + 1..text.len() - 1]);
            if let Some(operation) = arithmetic(name) {
                let a = self.operand(state, args.first().ok_or("MIR operand absent")?)?;
                if name == "Not" {
                    return Ok(Node::scalar(
                        &a.ty,
                        value::binary(
                            "xor",
                            &[],
                            a.value()?,
                            &value::constant("-1", a.value()?.width()?)?,
                        )?,
                    ));
                }
                if name == "Neg" {
                    let zero = value::constant("0", a.value()?.width()?)?;
                    let flags = if integer(&a.ty).is_some_and(|(_, s)| s) {
                        vec!["nsw"]
                    } else {
                        Vec::new()
                    };
                    let v = value::binary("sub", &flags, &zero, a.value()?)?;
                    self.obligation(state, &v.defined, "Rust negation overflow")?;
                    return Ok(Node::scalar(&a.ty, v));
                }
                let mut b = self.operand(state, args.get(1).ok_or("MIR second operand absent")?)?;
                if matches!(operation, "shl" | "lshr" | "ashr") {
                    let valid = format!(
                        "(bvult {} {})",
                        b.value()?.expr,
                        bv(u128::from(a.value()?.width()?), b.value()?.width()?)
                    );
                    self.obligation(state, &valid, "Rust shift amount out of range")?;
                    b = self.cast(b, &a.ty)?;
                }
                if matches!(operation, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
                    let prefix = if integer(&a.ty).is_some_and(|(_, s)| s) {
                        "s"
                    } else {
                        "u"
                    };
                    let predicate = if matches!(operation, "eq" | "ne") {
                        operation.to_owned()
                    } else {
                        format!("{prefix}{operation}")
                    };
                    return Ok(Node::scalar(
                        "bool",
                        value::compare(&predicate, false, a.value()?, b.value()?)?,
                    ));
                }
                if name.ends_with("WithOverflow") {
                    let flag = if integer(&a.ty).is_some_and(|(_, s)| s) {
                        "nsw"
                    } else {
                        "nuw"
                    };
                    let checked = value::binary(operation, &[flag], a.value()?, b.value()?)?;
                    let normal = value::binary(operation, &[], a.value()?, b.value()?)?;
                    let overflow = Value::bits(
                        format!("(ite {} {} {})", checked.defined, bv(0, 1), bv(1, 1)),
                        1,
                        "true".into(),
                    );
                    return Ok(Node {
                        ty: expected.into(),
                        kind: Data::Tuple(vec![
                            Node::scalar(&a.ty, normal),
                            Node::scalar("bool", overflow),
                        ]),
                    });
                }
                if matches!(operation, "udiv" | "urem") {
                    if integer(&a.ty).is_some_and(|(_, s)| s) {
                        return Err("unsupported MIR signed division".into());
                    }
                    self.obligation(
                        state,
                        &format!(
                            "(not (= {} {}))",
                            b.value()?.expr,
                            bv(0, b.value()?.width()?)
                        ),
                        "Rust zero divisor",
                    )?;
                }
                let operation = if operation == "lshr" && integer(&a.ty).is_some_and(|(_, s)| s) {
                    "ashr"
                } else {
                    operation
                };
                return Ok(Node::scalar(
                    &a.ty,
                    value::binary(operation, &[], a.value()?, b.value()?)?,
                ));
            }
            if let Some(definition) = self.program.registry.definition(expected).cloned()
                && definition.variants.is_empty()
                && definition.supported
                && super::parse::base_type(name) == super::parse::base_type(expected)
            {
                let values = args
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| self.operand(state, s))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != definition.fields.len() {
                    return Err("MIR tuple struct arity differs".into());
                }
                return Ok(Node {
                    ty: expected.into(),
                    kind: Data::Struct(values),
                });
            }
            if let Some((_, variant)) = name.rsplit_once("::")
                && let Some(tag) = self.program.registry.variant(expected, variant)
            {
                let fields = args
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| self.operand(state, s))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Node {
                    ty: expected.into(),
                    kind: Data::Enum {
                        variant: variant.into(),
                        tag,
                        fields,
                    },
                });
            }
        }
        if let Some((_, variant)) = text.rsplit_once("::")
            && let Some(tag) = self.program.registry.variant(expected, variant)
        {
            return Ok(Node {
                ty: expected.into(),
                kind: Data::Enum {
                    variant: variant.into(),
                    tag,
                    fields: Vec::new(),
                },
            });
        }
        Err(format!("unsupported MIR rvalue {text}"))
    }
    pub fn cast(&self, node: Node, target: &str) -> Result<Node, String> {
        let (old, signed) = integer(&node.ty).ok_or("unsupported MIR cast source type")?;
        let (new, _) = integer(target).ok_or("unsupported MIR cast target type")?;
        if target == "bool" && node.ty != "bool" {
            return Err("MIR integer-to-bool cast unsupported".into());
        }
        let value = node.value()?;
        let expr = if new == old {
            value.expr.clone()
        } else if new < old {
            format!("((_ extract {} 0) {})", new - 1, value.expr)
        } else {
            format!(
                "((_ {}_extend {}) {})",
                if signed { "sign" } else { "zero" },
                new - old,
                value.expr
            )
        };
        Ok(Node::scalar(
            target,
            Value::bits(expr, new, value.defined.clone()),
        ))
    }
    pub fn bind(&mut self, state: &mut State, node: Node) -> Result<Node, String> {
        match node.kind {
            Data::Scalar(value) => {
                let expr = self
                    .core
                    .symbol(&format!("(_ BitVec {})", value.width()?))?;
                state.constraints.push(format!("(= {expr} {})", value.expr));
                Ok(Node::scalar(
                    &node.ty,
                    Value::bits(expr, value.width()?, value.defined),
                ))
            }
            Data::Tuple(values) | Data::Array(values) | Data::Struct(values) => {
                let kind_tag = if node.ty.starts_with('[') {
                    0
                } else if node.ty.starts_with('(') {
                    1
                } else {
                    2
                };
                let fields = values
                    .into_iter()
                    .map(|v| self.bind(state, v))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node {
                    ty: node.ty,
                    kind: match kind_tag {
                        0 => Data::Array(fields),
                        1 => Data::Tuple(fields),
                        _ => Data::Struct(fields),
                    },
                })
            }
            other => Ok(Node {
                ty: node.ty,
                kind: other,
            }),
        }
    }
}

fn arithmetic(name: &str) -> Option<&'static str> {
    Some(match name {
        "Add" | "AddWithOverflow" => "add",
        "Sub" | "SubWithOverflow" => "sub",
        "Mul" | "MulWithOverflow" => "mul",
        "BitAnd" => "and",
        "BitOr" => "or",
        "BitXor" => "xor",
        "Shl" => "shl",
        "Shr" => "lshr",
        "Div" => "udiv",
        "Rem" => "urem",
        "Eq" => "eq",
        "Ne" => "ne",
        "Lt" => "lt",
        "Le" => "le",
        "Gt" => "gt",
        "Ge" => "ge",
        "Not" => "not",
        "Neg" => "neg",
        _ => return None,
    })
}

// MIR prints string constants with Rust escapes; unsupported escapes fail.
fn decode(text: &str) -> Result<Vec<u8>, String> {
    let mut chars = text.chars();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next().ok_or("MIR short string escape")? {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            '0' => result.push('\0'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            'u' => {
                if chars.next() != Some('{') {
                    return Err("MIR Unicode escape invalid".into());
                }
                let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                let code =
                    u32::from_str_radix(&hex, 16).map_err(|_| "MIR Unicode escape invalid")?;
                result.push(char::from_u32(code).ok_or("MIR Unicode scalar invalid")?);
            }
            _ => return Err("unsupported MIR string escape".into()),
        }
    }
    Ok(result.into_bytes())
}
