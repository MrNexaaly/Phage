//! Semantics for the supported LLVM instruction subset. Unsupported flags,
//! types and calls stop analysis of that feasible path with an unknown result.

use crate::{
    engine::{Engine, State, Step},
    ir::{Line, split, typed},
    memory,
    value::{self, Kind, Value, and, bv, truth},
};

impl Engine {
    pub fn execute(&mut self, state: &mut State, line: &Line) -> Result<Step, String> {
        let (destination, code) = line
            .text
            .split_once(" = ")
            .map_or((None, line.text.as_str()), |(name, rest)| {
                (Some(name), rest)
            });
        // One thread runs the property: atomic orderings have no effect.
        let atomic: String;
        let code = if code.starts_with("load atomic ") || code.starts_with("store atomic ") {
            atomic = crate::atomics::plain_access(code)?;
            atomic.as_str()
        } else if ["atomicrmw ", "cmpxchg ", "fence "]
            .iter()
            .any(|p| code.starts_with(p))
        {
            atomic = crate::atomics::without_orderings(code);
            atomic.as_str()
        } else {
            code
        };
        let opcode = code.split_whitespace().next().ok_or("empty instruction")?;
        if code.contains('@') {
            self.globals_for(state, code)?;
        }
        if code.contains("inttoptr (i64 ") {
            self.constant_pointers(state, code);
        }
        let rest = code[opcode.len()..].trim();
        // A load of bytes derived from the random source (see opaque.rs).
        let mut derived = false;
        let value = match opcode {
            "alloca" => {
                let args = split(rest);
                let ty = args[0];
                let size =
                    if let Some(width) = ty.strip_prefix('i').and_then(|w| w.parse::<u64>().ok()) {
                        width.div_ceil(8)
                    } else if ty == "ptr" {
                        8
                    } else if let Some((e, s)) = crate::floats::format(ty) {
                        u64::from(e + s) / 8
                    } else if ty.starts_with('[') && ty.ends_with(" x i8]") {
                        ty[1..]
                            .split_whitespace()
                            .next()
                            .ok_or("allocation count missing")?
                            .parse::<u64>()
                            .map_err(|_| "invalid allocation count")?
                    } else {
                        return Err(format!("unsupported allocation type {ty}"));
                    };
                if args.iter().skip(1).any(|arg| !arg.starts_with("align ")) {
                    return Err("dynamic allocation count is unsupported".into());
                }
                // Omitted alignment is the type's ABI alignment.
                let alignment = align(&args, ty).unwrap_or(1);
                if size > 4096 {
                    return Err("stack allocation exceeds prototype limit".into());
                }
                let object = format!("object{}", self.fresh);
                let bytes = self.symbol("(Array (_ BitVec 64) (_ BitVec 8))")?;
                state.locals.push(object.clone());
                let managed =
                    self.stack_managed(state, destination.ok_or("alloca result missing")?)?;
                let value = self.allocation_at(
                    state,
                    object.clone(),
                    (size, alignment),
                    bytes,
                    false,
                    !managed,
                )?;
                let allocated = state
                    .memory
                    .get_mut(&object)
                    .ok_or("stack allocation missing")?;
                allocated.stack = true;
                allocated.managed = managed;
                allocated.alive = !managed;
                value
            }
            "store" => {
                let args = split(rest);
                if args.len() < 2 {
                    return Err("store operands missing".into());
                }
                // Storing undef leaves undefined bytes (undef, not poison).
                let mut data = crate::operands::observe(self.typed_transfer(state, args[0])?)?;
                let (data_type, _) = typed(args[0])?;
                if crate::vectors::vector_type(data_type).is_some() {
                    let pointer = self.typed_operand(state, args[1])?;
                    let pointer = self.dereference(state, &pointer)?;
                    let alignment = align(&args, data_type)?;
                    self.vector_store(state, data_type, &data, &pointer, alignment)?;
                    return Ok(Step::Continue);
                }
                if matches!(data.kind, Kind::Float(..)) {
                    data = Self::float_bits(&data);
                }
                if let Kind::PointerBits(copied) = &data.kind {
                    // Integer bytes that are an exact pointer copy keep its
                    // provenance, as a memcpy of the same bytes would.
                    data = Value {
                        defined: and(&[copied.defined.clone(), data.defined.clone()]),
                        ..(**copied).clone()
                    };
                }
                let pointer = self.typed_operand(state, args[1])?;
                let pointer = self.dereference(state, &pointer)?;
                let alignment = align(&args, typed(args[0])?.0)?;
                let is_pointer =
                    matches!(data.kind, Kind::Pointer { .. } | Kind::PointerChoice { .. });
                let bytes = if is_pointer {
                    8
                } else {
                    u64::from(data.width()?.div_ceil(8))
                };
                let valid = memory::access_store(&state.memory, &pointer, bytes, alignment)?;
                self.safety(state, &valid, "invalid memory write")?;
                if is_pointer {
                    self.store_pointer(state, &pointer, &data)?;
                } else {
                    // A later pointer load re-proves its bytes, so overwriting
                    // a pointer slot needs no bookkeeping here.
                    memory::store(&mut state.memory, &pointer, &data)?;
                }
                let tainted = self.is_tainted(&data.expr);
                self.mark_taint(state, &pointer, bytes, tainted)?;
                return Ok(Step::Continue);
            }
            "load" => {
                let (loaded, tainted) = self.load_instruction(state, rest, line.noundef)?;
                derived = tainted;
                loaded
            }
            "getelementptr" => {
                let args = split(rest);
                let (flags, ty) = gep_type(args[0])?;
                if flags
                    .iter()
                    .any(|flag| !matches!(*flag, "inbounds" | "nuw"))
                {
                    return Err("unsupported GEP flag".into());
                }
                let inbounds = flags.contains(&"inbounds");
                let nuw = flags.contains(&"nuw");
                let module = self
                    .modules
                    .iter()
                    .find(|m| m.name == state.module)
                    .cloned()
                    .ok_or("current module is not loaded")?;
                let pointer = self.typed_operand(state, args[1])?;
                // LangRef: inbounds makes every index scaling `mul nsw` and the
                // running offset sum `add nsw`; nuw makes them `mul nuw` and
                // `add nuw`. Any wrap makes the result poison.
                let mut flags = Vec::new();
                if inbounds {
                    flags.push("nsw");
                }
                if nuw {
                    flags.push("nuw");
                }
                let checked = |op: &str, a: &Value, b: &Value| -> Result<Value, String> {
                    let mut result = value::binary(op, &[], a, b)?;
                    for flag in &flags {
                        let strict = value::binary(op, &[flag], a, b)?;
                        result.defined = and(&[result.defined, strict.defined]);
                    }
                    Ok(result)
                };
                let scaled = |index: Value, size: u64| -> Result<Value, String> {
                    if index.width()? != 64 {
                        return Err("GEP requires a 64-bit index".into());
                    }
                    if size == 1 {
                        return Ok(index);
                    }
                    checked("mul", &index, &value::constant(&size.to_string(), 64)?)
                };
                let size = |ty: &str| {
                    crate::constants::size_of(ty, &module.types).ok_or("GEP element size is unknown")
                };
                // One index scales by the element size; an array type takes a
                // leading zero index and scales the second by its element.
                // These two shapes keep their original single-term formula.
                let offset = match (args.len(), ty.strip_prefix('[')) {
                    (3, _) => scaled(self.typed_operand(state, args[2])?, size(ty)?)?,
                    (4, Some(array)) if args[2].trim() == "i64 0" => {
                        let element = array
                            .split_once(" x ")
                            .map(|(_, e)| e.trim_end_matches(']').trim())
                            .ok_or("GEP array type malformed")?;
                        scaled(self.typed_operand(state, args[3])?, size(element)?)?
                    }
                    // General form (LLVM 21 emits `%T, ptr %p, i64 %i, i32 1`
                    // for a field of an array element): the first index
                    // scales by the source type, then each index steps into an
                    // array element (scaled) or a struct field (constant).
                    _ => {
                        let mut current = ty;
                        let mut total = scaled(self.typed_operand(state, args[2])?, size(ty)?)?;
                        for index in &args[3..] {
                            let term = if let Some(element) =
                                crate::constants::array_element(current, &module.types)
                            {
                                current = element;
                                scaled(self.typed_operand(state, index)?, size(element)?)?
                            } else {
                                let field = index
                                    .trim()
                                    .strip_prefix("i32 ")
                                    .and_then(|n| n.trim().parse::<usize>().ok())
                                    .ok_or("GEP struct index must be a constant i32")?;
                                let (field_offset, field_ty) =
                                    crate::constants::field_offset(current, field, &module.types)
                                        .ok_or("GEP indexes a non-aggregate or missing field")?;
                                current = field_ty;
                                value::constant(&field_offset.to_string(), 64)?
                            };
                            total = checked("add", &total, &term)?;
                        }
                        total
                    }
                };
                if pointer.defined == "false" || offset.defined == "false" {
                    crate::pointers::without_provenance(
                        format!("(bvadd {} {})", pointer.expr, offset.expr),
                        "false".into(),
                    )
                } else {
                    let pointer = self.resolve_pointer(state, &pointer)?;
                    memory::gep(&state.memory, &pointer, &offset, inbounds, nuw)?
                }
            }
            "add" | "sub" | "mul" | "and" | "or" | "xor" | "shl" | "lshr" | "ashr" | "udiv"
            | "urem" | "sdiv" | "srem" | "icmp"
                if crate::vectors::split_vector(split(rest)[0]).is_some() =>
            {
                self.vector_lanewise(state, opcode, &split(rest))?
            }
            "fadd" | "fsub" | "fmul" | "fdiv" | "frem" | "fneg" | "fcmp" | "uitofp" | "sitofp"
            | "fptoui" | "fptosi" | "fpext" | "fptrunc" => self
                .float_instruction(state, opcode, rest)?
                .ok_or("unsupported floating-point instruction")?,
            "bitcast" => {
                let (source, target) = rest.split_once(" to ").ok_or("bitcast target missing")?;
                let (source_type, _) = typed(source)?;
                let value = self.typed_operand(state, source)?;
                if source_type == "ptr" && target == "ptr" {
                    let name = destination.ok_or("bitcast result missing")?;
                    state.env.insert(name.to_owned(), value);
                    return Ok(Step::Continue);
                }
                match (
                    crate::floats::format(source_type),
                    crate::floats::format(target),
                ) {
                    (Some(_), None) if integer_width(target).is_ok() => Self::float_bits(&value),
                    (None, Some(format)) if integer_width(source_type).is_ok() => {
                        self.bits_float(&value, format)?
                    }
                    (Some(a), Some(b)) if a == b => value,
                    (None, None) => crate::vectors::bitcast(&value, source_type, target)?,
                    _ => return Err(format!("unsupported bitcast {source_type} to {target}")),
                }
            }
            "insertelement" | "extractelement" | "shufflevector" => {
                self.vector_element(state, opcode, rest)?
            }
            "add" | "sub" | "mul" | "and" | "or" | "xor" | "shl" | "lshr" | "ashr" | "udiv"
            | "urem" | "sdiv" | "srem" => {
                let args = split(rest);
                if args.len() != 2 {
                    return Err("binary operands missing".into());
                }
                let words: Vec<_> = args[0].split_whitespace().collect();
                let type_index = words
                    .iter()
                    .position(|w| w.starts_with('i') && w[1..].parse::<u32>().is_ok())
                    .ok_or("binary type missing")?;
                let ty = words[type_index];
                let left = self.transfer_operand(
                    state,
                    words.get(type_index + 1).ok_or("left operand missing")?,
                    ty,
                )?;
                let right = self.transfer_operand(state, args[1], ty)?;
                if let Some(result) =
                    self.undef_bitwise(state, opcode, &words[..type_index], &left, &right)?
                {
                    result
                } else {
                    let left = crate::operands::observe(left)?;
                    let right = crate::operands::observe(right)?;
                    let (left, right) = crate::canonical::commute(opcode, left, right);
                    if matches!(opcode, "udiv" | "urem" | "sdiv" | "srem") {
                        // LLVM LangRef: zero division (and signed INT_MIN / -1) is
                        // immediate UB, unlike arithmetic poison that may stay unobserved.
                        let width = right.width()?;
                        let mut valid = vec![
                            right.defined.clone(),
                            format!("(not (= {} {}))", right.expr, bv(0, width)),
                        ];
                        if opcode.starts_with('s') {
                            // A poison dividend only poisons the result.
                            valid.push(format!(
                                "(or (not {}) {})",
                                left.defined,
                                value::signed_division_defined(&left, &right, width)
                            ));
                        }
                        self.safety(state, &and(&valid), "zero, overflowing or poison divisor")?;
                    }
                    let flags = &words[..type_index];
                    let result = value::binary(opcode, flags, &left, &right)?;
                    let result = crate::partial::binary(opcode, flags, &left, &right, result)?;
                    match self.canonical_xor(state, opcode, flags, &left, &right, &result)? {
                        Some(canonical) => canonical,
                        None => {
                            self.opaque_binary(state, opcode, flags, &left, &right, result)?
                                .0
                        }
                    }
                }
            }
            "icmp" => self.compare_instruction(state, rest)?,
            "inttoptr" => {
                let (source, target) = rest.split_once(" to ").ok_or("inttoptr target missing")?;
                if target != "ptr" {
                    return Err("only inttoptr to ptr is supported".into());
                }
                let mut address = self.typed_operand(state, source)?;
                let width = address.width()?;
                if width != 64 {
                    let (cast, expr) = if width < 64 {
                        (
                            "zext",
                            format!("((_ zero_extend {}) {})", 64 - width, address.expr),
                        )
                    } else {
                        ("trunc", format!("((_ extract 63 0) {})", address.expr))
                    };
                    address = crate::partial::cast(
                        cast,
                        &address,
                        64,
                        Value::bits(expr, 64, address.defined.clone()),
                    )?;
                }
                if let Kind::PointerBits(original) = &address.kind {
                    Value {
                        defined: and(&[original.defined.clone(), address.defined.clone()]),
                        ..(**original).clone()
                    }
                } else if let Some(pointer) = self.exposed(state, &address) {
                    pointer
                } else {
                    match self
                        .concrete(state, &address.expr)?
                        .and_then(|n| u64::try_from(n).ok())
                    {
                        Some(bits) => Value {
                            defined: address.defined.clone(),
                            ..self.dangling(state, bits)
                        },
                        // No provenance: the address can be compared or cast
                        // back, but no object is reachable through it (any
                        // access stays unknown).
                        None => crate::pointers::without_provenance(address.expr, address.defined),
                    }
                }
            }
            "insertvalue" => {
                let args = split(rest);
                if args.len() < 3 {
                    return Err("insertvalue operands missing".into());
                }
                let (ty, base) = typed(args[0])?;
                let aggregate = self.transfer_operand(state, base, ty)?;
                let element = self.typed_transfer(state, args[1])?;
                let path = indices(&args[2..])?;
                value::insert(aggregate, &path, element)?
            }
            "ptrtoint" => {
                let (source, target) = rest.split_once(" to ").ok_or("ptrtoint target missing")?;
                if target != "i64" {
                    return Err("only 64-bit ptrtoint is supported".into());
                }
                let pointer = self.typed_operand(state, source)?;
                self.pointer_integer(pointer)?
            }
            "zext" | "sext" | "trunc" => {
                let (source, target) = rest.split_once(" to ").ok_or("cast target missing")?;
                let nneg = source.starts_with("nneg ");
                let source = source.strip_prefix("nneg ").unwrap_or(source);
                let source = self.typed_operand(state, source)?;
                let old = source.width()?;
                let new = integer_width(target)?;
                let expr = match opcode {
                    "zext" if new > old => {
                        format!("((_ zero_extend {}) {})", new - old, source.expr)
                    }
                    "sext" if new > old => {
                        format!("((_ sign_extend {}) {})", new - old, source.expr)
                    }
                    "trunc" if new < old => format!("((_ extract {} 0) {})", new - 1, source.expr),
                    _ => return Err("invalid integer cast widths".into()),
                };
                let mut valid = vec![source.defined.clone()];
                if nneg {
                    if opcode != "zext" {
                        return Err("unsupported nneg cast".into());
                    }
                    let bit = old - 1;
                    valid.push(format!(
                        "(= ((_ extract {bit} {bit}) {}) {})",
                        source.expr,
                        bv(0, 1)
                    ));
                }
                crate::partial::cast(opcode, &source, new, Value::bits(expr, new, and(&valid)))?
            }
            "select" => {
                let args = split(rest);
                if args.len() != 3 {
                    return Err("select operands missing".into());
                }
                let condition = self.typed_operand(state, args[0])?;
                let yes = self.typed_transfer(state, args[1])?;
                let no = self.typed_transfer(state, args[2])?;
                let cond = truth(&condition)?;
                let undef_condition =
                    matches!(self.typed_transfer(state, args[0])?.kind, Kind::Undef(_));
                crate::partial::select(&cond, &condition.defined, undef_condition, yes, no)?
            }
            "extractvalue" => {
                let args = split(rest);
                let (ty, operand) = typed(args[0])?;
                let value = self.transfer_operand(state, operand, ty)?;
                value::extract(value, &indices(&args[1..])?)?
            }
            "call" | "tail" => return self.call(state, opcode, rest, destination),
            "atomicrmw" => self.atomicrmw(state, rest)?,
            "cmpxchg" => self.cmpxchg(state, rest)?,
            "fence" => return Ok(Step::Continue),
            "switch" => {
                let (head, cases) = rest.split_once('[').ok_or("switch cases missing")?;
                let head = split(head.trim().trim_end_matches(','));
                let scrutinee =
                    self.typed_operand(state, head.first().ok_or("switch value missing")?)?;
                let width = scrutinee.width()?;
                let default = label(head.get(1).ok_or("switch default missing")?)?;
                let words: Vec<_> = cases
                    .trim()
                    .strip_suffix(']')
                    .ok_or("switch cases unterminated")?
                    .split_whitespace()
                    .collect();
                if words.len() % 4 != 0 {
                    return Err("malformed switch case list".into());
                }
                let mut targets = Vec::new();
                for case in words.chunks(4) {
                    if case[0] != format!("i{width}") || case[2] != "label" {
                        return Err("malformed switch case".into());
                    }
                    let value = value::constant(case[1].trim_end_matches(','), width)?;
                    targets.push((value.expr, label(&format!("label {}", case[3]))?));
                }
                return Ok(Step::Switch(scrutinee, targets, default));
            }
            "br" => {
                let args = split(rest);
                if args.len() == 1 {
                    return Ok(Step::Jump(label(args[0])?));
                }
                if args.len() == 3 {
                    return Ok(Step::Branch(
                        self.typed_operand(state, args[0])?,
                        label(args[1])?,
                        label(args[2])?,
                    ));
                }
                return Err("branch operands missing".into());
            }
            "ret" if rest == "void" => return Ok(Step::Return(None)),
            "ret" => return Ok(Step::Return(Some(self.typed_transfer(state, rest)?))),
            "unreachable" => {
                return Ok(Step::Panic("reachable LLVM unreachable instruction".into()));
            }
            _ => {
                return Err(format!(
                    "{} instruction {opcode}",
                    if matches!(
                        opcode,
                        "freeze"
                            | "indirectbr"
                            | "landingpad"
                            | "resume"
                            | "callbr"
                            | "va_arg"
                            | "catchpad"
                            | "cleanuppad"
                    ) {
                        "unsupported"
                    } else {
                        "unrecognized"
                    }
                ));
            }
        };
        let name = destination.ok_or("value-producing instruction has no destination")?;
        let opaque = value.expr.starts_with("(ru_");
        let value = self.materialize(state, value)?;
        if derived || opaque {
            self.taint(&value);
        }
        state.env.insert(name.to_owned(), value);
        Ok(Step::Continue)
    }
}

/// GEP flags and source element type from `inbounds nuw i64` and the like.
fn gep_type(text: &str) -> Result<(Vec<&str>, &str), String> {
    let text = text.trim();
    // The source element type is the last token: a bracketed struct, array
    // or vector type, a quoted named type, or a plain word.
    let start = if text.ends_with('"') {
        text.find("%\"")
    } else if text.ends_with([']', '}', '>']) {
        let mut depth = 0i32;
        text.bytes().enumerate().rev().find_map(|(i, byte)| {
            match byte {
                b']' | b'}' | b'>' => depth += 1,
                b'[' | b'{' | b'<' => depth -= 1,
                _ => {}
            }
            // A packed struct opens with `<{`.
            (depth == 0).then(|| {
                if i > 0 && text.as_bytes()[i - 1] == b'<' {
                    i - 1
                } else {
                    i
                }
            })
        })
    } else {
        None
    };
    let (flags, ty) = match start {
        Some(i) => (&text[..i], &text[i..]),
        None => text.rsplit_once(' ').unwrap_or(("", text)),
    };
    Ok((flags.split_whitespace().collect(), ty.trim()))
}

fn indices(args: &[&str]) -> Result<Vec<usize>, String> {
    args.iter()
        .map(|i| {
            i.trim()
                .parse()
                .map_err(|_| format!("invalid aggregate index {i}"))
        })
        .collect()
}
pub(crate) fn integer_width(text: &str) -> Result<u32, String> {
    text.trim()
        .strip_prefix('i')
        .and_then(|w| w.parse().ok())
        .filter(|w| *w > 0 && *w <= 128)
        .ok_or_else(|| format!("unsupported integer type {text}"))
}
fn label(text: &str) -> Result<String, String> {
    text.trim()
        .strip_prefix("label %")
        .map(str::to_owned)
        .ok_or_else(|| format!("unsupported branch label {text}"))
}
/// The access alignment: the stated one, else (LangRef) the ABI alignment
/// of the accessed type under the x86_64 data layout.
pub(crate) fn align(args: &[&str], ty: &str) -> Result<u64, String> {
    if let Some(stated) = args
        .iter()
        .find_map(|arg| arg.strip_prefix("align "))
        .and_then(|s| s.parse().ok())
    {
        return Ok(stated);
    }
    let ty = ty.trim();
    let bytes = if ty == "ptr" {
        8
    } else if let Some((e, s)) = crate::floats::format(ty) {
        u64::from(e + s) / 8
    } else if let Some((count, width)) = crate::vectors::vector_type(ty) {
        (count as u64 * u64::from(width)).div_ceil(8)
    } else {
        ty.strip_prefix('i')
            .and_then(|w| w.parse::<u64>().ok())
            .ok_or_else(|| format!("implicit alignment of {ty} is unsupported"))?
            .div_ceil(8)
    };
    // Vectors align to their size; integers wider than i128 to i128's 16.
    Ok(if ty.starts_with('<') {
        bytes.next_power_of_two()
    } else {
        bytes.next_power_of_two().min(16)
    })
}
