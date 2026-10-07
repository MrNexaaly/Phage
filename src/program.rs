//! The program being verified: the checked crate's module plus library
//! modules loaded on demand. Resolves callee bodies (private symbols first
//! in the caller's module), allocates referenced globals lazily (resolving
//! declarations to another module's definition, as the linker would) and
//! formats source positions.

use crate::{
    engine::{Engine, Library, State},
    ir::{Function, Global, Module},
    value::{Kind, Value},
};
use std::rc::Rc;

impl Engine {
    /// Body of `name` as seen from module `from`: its own definition (any
    /// linkage) first, else an exported definition of another module (the
    /// checked crate first), else the library index. Private and internal
    /// definitions never resolve across modules.
    pub fn function(&mut self, name: &str, from: &str) -> Result<Option<Rc<Function>>, String> {
        let find = |modules: &[Rc<Module>]| {
            modules
                .iter()
                .find(|m| m.name == from && m.defines(name))
                .or_else(|| modules.iter().find(|m| m.exports(name)))
                .cloned()
        };
        let module = match find(&self.modules) {
            Some(module) => Some(module),
            None => self.library_module(name)?,
        };
        let Some(module) = module else {
            return Ok(None);
        };
        self.unambiguous(name, &module.name, from)?;
        let key = format!("{}\u{0}{name}", module.name);
        if let Some(function) = self.functions.get(&key) {
            return Ok(Some(function.clone()));
        }
        let function = Rc::new(module.function(name)?);
        self.functions.insert(key, function.clone());
        Ok(Some(function))
    }
    /// The sysroot module defining `symbol`, loaded (and its bitcode
    /// assumption recorded) on first use.
    fn library_module(&mut self, symbol: &str) -> Result<Option<Rc<Module>>, String> {
        if matches!(self.library, Library::Pending(_))
            && let Library::Pending(compiler) =
                std::mem::replace(&mut self.library, Library::Disabled)
        {
            self.library = match crate::library::Library::open(&compiler) {
                Ok(library) => Library::Open(library),
                Err(error) => Library::Failed(error),
            };
        }
        let Library::Open(library) = &mut self.library else {
            return Ok(None);
        };
        let Some(found) = library.module_for(symbol)? else {
            return Ok(None);
        };
        if !self.modules.iter().any(|m| m.name == found.name) {
            self.assumptions.insert(format!(
                "{} bodies come from the rustc sysroot's embedded LLVM bitcode",
                found.name
            ));
            self.modules.push(found.clone());
        }
        Ok(Some(found))
    }
    /// Cross-module resolution must respect the index's ambiguity marks
    /// even when the exporting module happens to be loaded already.
    fn unambiguous(&self, symbol: &str, module: &str, from: &str) -> Result<(), String> {
        if module != from
            && let Library::Open(library) = &self.library
            && library.is_ambiguous(symbol)
        {
            return Err(format!(
                "{symbol} is defined incompatibly by several sysroot crates"
            ));
        }
        Ok(())
    }
    /// The already-resolved body a state is executing.
    pub fn current(&self, state: &State) -> Result<Rc<Function>, String> {
        self.functions
            .get(&format!("{}\u{0}{}", state.module, state.function))
            .cloned()
            .ok_or_else(|| format!("function {} is not resolved", state.function))
    }
    /// True when `callee`, as seen from module `from`, carries `noreturn`:
    /// the caller's own declaration or definition decides, otherwise the
    /// definition that the call resolves to.
    pub fn diverges(&mut self, callee: &str, from: &str) -> Result<bool, String> {
        if let Some(module) = self.modules.iter().find(|m| m.name == from) {
            if module.names_function(callee) {
                return Ok(module.noreturn.contains(callee));
            }
        } else if self.noreturn.contains(callee) {
            return Ok(true);
        }
        let Some(function) = self.function(callee, from)? else {
            return Ok(false);
        };
        Ok(self
            .modules
            .iter()
            .find(|m| m.name == function.module)
            .is_some_and(|m| m.noreturn.contains(callee)))
    }
    /// Memory key of a global: user globals keep their name, library
    /// globals are qualified by module so private names cannot collide.
    pub fn qualify(module: &str, name: &str) -> String {
        if module == "user" {
            name.to_owned()
        } else {
            format!("{module}::{name}")
        }
    }
    /// Allocates globals and function code objects that `text` uses as
    /// values (a symbol directly followed by `(` is a callee, not a value).
    pub fn globals_for(&mut self, state: &mut State, text: &str) -> Result<(), String> {
        let mut rest = text;
        while let Some(at) = rest.find('@') {
            let tail = &rest[at..];
            let length = if let Some(quoted) = tail.strip_prefix("@\"") {
                quoted.find('"').map_or(tail.len(), |q| q + 3)
            } else {
                tail[1..]
                    .find(|c: char| !(c.is_ascii_alphanumeric() || "_.$-".contains(c)))
                    .map_or(tail.len(), |n| n + 1)
            };
            let symbol = &tail[..length];
            rest = &tail[length..];
            if symbol.len() > 1 && !rest.starts_with('(') {
                let module = state.module.clone();
                // Unknown symbols stay unresolved and fail only when used.
                let _ = self.symbol_pointer(state, &module, symbol);
            }
        }
        Ok(())
    }

    /// A pointer to global or function `symbol` as referenced from `module`,
    /// allocating the global (with its relocations) or the function's code
    /// object first. A global the module only declares resolves to the
    /// exported definition of another module, as the linker would.
    pub fn symbol_pointer(
        &mut self,
        state: &mut State,
        module: &str,
        symbol: &str,
    ) -> Result<Value, String> {
        let reference = Self::qualify(module, symbol);
        let key = self
            .global_keys
            .get(&reference)
            .cloned()
            .unwrap_or_else(|| reference.clone());
        if !state.memory.contains_key(&key) {
            let source = self
                .modules
                .iter()
                .find(|m| m.name == module)
                .cloned()
                .ok_or_else(|| format!("module {module} is not loaded"))?;
            let bare = symbol.trim_start_matches('@').trim_matches('"');
            if let Some(global) = source.globals.get(bare) {
                self.global(state, module, global)?;
            } else if source.names_function(bare) {
                if source.weak.contains(bare) {
                    if bare != "getrandom" {
                        return Err(format!("address of weak function {bare} is unsupported"));
                    }
                    self.assumptions.insert(
                        "the OS random source (getrandom) is available at link time".into(),
                    );
                }
                // Code objects have size zero and no distinctness assumption:
                // unnamed_addr functions may be merged by the linker.
                self.allocation(
                    state,
                    key.clone(),
                    0,
                    1,
                    "((as const (Array (_ BitVec 64) (_ BitVec 8))) (_ bv0 8))".into(),
                    true,
                )?;
                let object = state.memory.get_mut(&key).ok_or("code object missing")?;
                object.function = Some((module.to_owned(), bare.to_owned()));
            } else if source.global_definitions.contains(bare) {
                return Err(format!("unsupported global definition {symbol}"));
            } else {
                let (owner, global) = self
                    .external_global(bare, module)?
                    .ok_or_else(|| format!("unsupported pointer target {symbol}"))?;
                let key = Self::qualify(&owner, &global.name);
                if !state.memory.contains_key(&key) {
                    self.global(state, &owner, &global)?;
                }
                self.global_keys.insert(reference, key.clone());
                return self.symbol_pointer(state, &owner, &global.name);
            }
        }
        let object = state.memory.get(&key).ok_or("global missing")?;
        Ok(Value {
            expr: object.base.clone(),
            kind: Kind::Pointer {
                object: key,
                offset: crate::value::bv(0, 64),
                bounded: true,
            },
            defined: "true".into(),
        })
    }

    /// The exported definition of global `symbol` in another loaded module
    /// or the sysroot, for a reference from `from`.
    fn external_global(
        &mut self,
        symbol: &str,
        from: &str,
    ) -> Result<Option<(String, Global)>, String> {
        let exported = |module: &Module| {
            module
                .globals
                .get(symbol)
                .filter(|g| !g.local)
                .map(|g| (module.name.clone(), g.clone()))
        };
        let found = match self
            .modules
            .iter()
            .filter(|m| m.name != from)
            .find_map(|m| exported(m))
        {
            Some(found) => Some(found),
            None => self
                .library_module(symbol)?
                .and_then(|module| exported(&module)),
        };
        if let Some((owner, _)) = &found {
            self.unambiguous(symbol, owner, from)?;
        }
        Ok(found)
    }

    /// Allocates a global; relocations in its initializer become stored
    /// pointers.
    pub fn global(
        &mut self,
        state: &mut State,
        module: &str,
        global: &Global,
    ) -> Result<(), String> {
        let key = Self::qualify(module, &global.name);
        let mut bytes = "((as const (Array (_ BitVec 64) (_ BitVec 8))) (_ bv0 8))".to_owned();
        for (i, byte) in global.bytes.iter().enumerate() {
            bytes = format!(
                "(store {bytes} {} {})",
                crate::value::bv(i as u128, 64),
                crate::value::bv(u128::from(*byte), 8)
            );
        }
        self.allocation(
            state,
            key.clone(),
            global.bytes.len() as u64,
            global.alignment,
            bytes,
            !global.mutable,
        )?;
        if global.mutable {
            // Writable allocations start uninitialized; a global starts at
            // its initializer (padding and `undef` parts are re-marked below).
            let object = state.memory.get_mut(&key).ok_or("global missing")?;
            object.initialized = crate::memory::ALL_DEFINED.into();
            object.known = None;
            self.assumptions.insert(
                "mutable statics start at their initializers: a fresh program, as in Kani".into(),
            );
        }
        if global.thread_local {
            self.assumptions
                .insert("one thread: a thread-local is the global's own copy".into());
        }
        if let Err(error) = self.relocate(state, module, &key, global) {
            // Transactional: a global whose pointers cannot all be resolved
            // must not stay allocated with placeholder bytes.
            state.memory.remove(&key);
            return Err(error);
        }
        Ok(())
    }

    fn relocate(
        &mut self,
        state: &mut State,
        module: &str,
        key: &str,
        global: &Global,
    ) -> Result<(), String> {
        {
            let this = state.memory.get_mut(key).ok_or("global missing")?;
            for offset in &global.padding {
                let at = crate::value::bv(u128::from(*offset), 64);
                this.set_flags(&at, Some(*offset), "false", "false");
            }
        }
        for relocation in &global.relocations {
            // The global is already allocated, so reference cycles terminate.
            let target = self.symbol_pointer(state, module, &relocation.symbol)?;
            let Kind::Pointer { object, .. } = &target.kind else {
                return Err("relocation target is not an object".into());
            };
            let size = state
                .memory
                .get(object)
                .ok_or("relocation target missing")?
                .size
                .upper();
            // An inbounds constant GEP past its target is a poison pointer.
            let defined = !relocation.inbounds || relocation.addend <= size;
            let offset = crate::value::bv(u128::from(relocation.addend), 64);
            let pointer = Value {
                expr: format!("(bvadd {} {offset})", target.expr),
                kind: Kind::Pointer {
                    object: object.clone(),
                    offset,
                    bounded: relocation.inbounds && defined,
                },
                defined: defined.to_string(),
            };
            let this = state.memory.get_mut(key).ok_or("global missing")?;
            for byte in 0..8u64 {
                let at = crate::value::bv(u128::from(relocation.offset + byte), 64);
                let low = byte * 8;
                this.bytes = format!(
                    "(store {} {at} ((_ extract {} {low}) {}))",
                    this.bytes,
                    low + 7,
                    pointer.expr
                );
                // An out-of-bounds constant GEP is a poison pointer.
                this.set_flags(
                    &at,
                    Some(relocation.offset + byte),
                    &defined.to_string(),
                    &(!defined).to_string(),
                );
            }
            this.pointers.push(crate::memory::PointerSlot {
                offset: crate::value::bv(u128::from(relocation.offset), 64),
                value: pointer,
                valid: "true".into(),
            });
        }
        Ok(())
    }
    /// Source position used in verdict details.
    pub fn at(state: &State, line: usize) -> String {
        if state.module == "user" {
            format!("LLVM line {line}")
        } else {
            format!("{} LLVM line {line}", state.module)
        }
    }
}
