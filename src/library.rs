//! Standard library bodies for calls the checked crate does not define.
//! Every sysroot rlib embeds its LLVM bitcode; it is disassembled once per
//! rustc commit into a cache with a symbol index, and a crate's module is
//! loaded only when one of its functions is first called. Shared generic
//! instances live in whichever sysroot crate emitted them (Vec<u8>::drop is
//! in gimli), so all crates are indexed. Missing tools or bitcode leave
//! calls unresolved, which stays unknown.

use crate::ir::Module;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};

const INDEX: &str = "index.tsv";
/// Part of the cache identity: a new index layout gets a new directory.
const FORMAT: &str = "index 4: matched compiler, tools and SHA-256 manifest";
/// Index marker for a symbol defined incompatibly by several crates.
const AMBIGUOUS: &str = "?";

pub struct Library {
    directory: PathBuf,
    /// Defined function or global symbol to the crate module defining it.
    index: BTreeMap<String, String>,
    loaded: BTreeMap<String, Rc<Module>>,
    hashes: BTreeMap<String, String>,
}

impl Library {
    /// Opens the cache for the active rustc, building it on first use.
    pub fn open(compiler: &crate::toolchain::Toolchain) -> Result<Self, String> {
        let version = &compiler.version;
        let commit = &compiler.commit;
        let libraries = &compiler.libraries;
        // The cache is keyed by the exact rlibs, not only the commit: two
        // sysroots of one commit may hold differently built libraries.
        let mut identity = format!(
            "{FORMAT}\n{version}\n{}\n{}\n",
            compiler.compiler.display(),
            libraries.display()
        );
        let mut rlibs = rlibs(libraries)?;
        rlibs.sort();
        for rlib in &rlibs {
            let meta = fs::metadata(rlib).map_err(|e| e.to_string())?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_nanos());
            identity.push_str(&format!(
                "{} {} {modified} {}\n",
                rlib.display(),
                meta.len(),
                crate::cache::hash(&fs::read(rlib).map_err(|e| e.to_string())?)?
            ));
        }
        let directory = cache_root()?
            .join("std")
            .join(format!("{commit}-{:016x}", fnv1a(identity.as_bytes())));
        if !directory.join(INDEX).is_file() {
            build(&directory, &rlibs, compiler, &identity)?;
        }
        let hashes = crate::cache::verify(&directory, &identity)?;
        let index = String::from_utf8(crate::cache::read(&directory, INDEX, &hashes)?)
            .map_err(|e| format!("library index: {e}"))?
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .map(|(symbol, module)| (symbol.to_owned(), module.to_owned()))
            .collect();
        Ok(Self {
            directory,
            index,
            loaded: BTreeMap::new(),
            hashes,
        })
    }

    pub fn is_ambiguous(&self, symbol: &str) -> bool {
        self.index.get(symbol).is_some_and(|m| m == AMBIGUOUS)
    }

    /// The module defining `symbol`, loading and indexing it on first use.
    pub fn module_for(&mut self, symbol: &str) -> Result<Option<Rc<Module>>, String> {
        let Some(name) = self.index.get(symbol).cloned() else {
            return Ok(None);
        };
        if name == AMBIGUOUS {
            return Err(format!(
                "{symbol} is defined incompatibly by several sysroot crates"
            ));
        }
        if let Some(module) = self.loaded.get(&name) {
            return Ok(Some(module.clone()));
        }
        let text = String::from_utf8(crate::cache::read(
            &self.directory,
            &format!("{name}.ll"),
            &self.hashes,
        )?)
        .map_err(|e| format!("library {name}: {e}"))?;
        let module = Rc::new(Module::new(&name, text, false)?);
        self.loaded.insert(name, module.clone());
        Ok(Some(module))
    }
}

/// Directory holding cached library modules.
pub fn cache_root() -> Result<PathBuf, String> {
    if let Some(path) = crate::setting("CACHE") {
        return Ok(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .ok_or("no cache directory: set PHAGE_CACHE")?;
    Ok(base.join("phage"))
}

fn rlibs(libraries: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(fs::read_dir(libraries)
        .map_err(|e| format!("{}: {e}", libraries.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rlib"))
        .collect())
}

/// Index rows `symbol<TAB>crate` from (symbol, crate, ODR linkage,
/// available_externally copy). A symbol exported by several crates is usable
/// only when every copy has ODR linkage (interchangeable); otherwise the
/// final link decides, which the cache does not model: ambiguous. A real
/// definition is preferred over an available_externally copy.
fn index_lines(mut index: Vec<(String, String, bool, bool)>) -> Vec<String> {
    index.sort();
    index
        .chunk_by(|a, b| a.0 == b.0)
        .map(|group| {
            let odr = group.iter().all(|row| row.2);
            let module = if group.len() == 1 || odr {
                group
                    .iter()
                    .find(|row| !row.3)
                    .unwrap_or(&group[0])
                    .1
                    .as_str()
            } else {
                AMBIGUOUS
            };
            format!("{}\t{module}", group[0].0)
        })
        .collect()
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Builds a complete cache generation in a private directory and publishes
/// it with one atomic rename; a concurrent builder's generation is kept.
struct Tools {
    ar: PathBuf,
    objcopy: PathBuf,
    link: PathBuf,
    dis: PathBuf,
}

fn build(
    directory: &Path,
    rlibs: &[PathBuf],
    compiler: &crate::toolchain::Toolchain,
    identity: &str,
) -> Result<(), String> {
    let tools = Tools {
        ar: compiler.tool("llvm-ar")?,
        objcopy: compiler.tool("llvm-objcopy")?,
        link: compiler.tool("llvm-link")?,
        dis: compiler.tool("llvm-dis")?,
    };
    let work = crate::cache::staging(directory)?;
    let result = (|| -> Result<Vec<String>, String> {
        let mut index: Vec<(String, String, bool, bool)> = Vec::new();
        for rlib in rlibs {
            let stem = rlib
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let name = stem
                .strip_prefix("lib")
                .unwrap_or(stem)
                .split('-')
                .next()
                .unwrap_or(stem)
                .to_owned();
            let scratch = work.join(format!("unpack-{name}"));
            let text = disassemble(rlib, &scratch, &tools)?;
            let _ = fs::remove_dir_all(&scratch);
            let Some(text) = text else {
                continue;
            };
            for line in text.lines() {
                // Module-local definitions can never be the target of a
                // reference from another crate, so they are not indexed.
                let symbol = if line.starts_with("define ") {
                    define_symbol(line)
                } else if line.starts_with('@') {
                    crate::ir::global_definition(line)
                } else {
                    None
                };
                if let Some(symbol) = symbol
                    && !crate::ir::local_linkage(line)
                {
                    index.push((
                        symbol.to_owned(),
                        name.clone(),
                        crate::ir::odr_linkage(line),
                        crate::ir::available_externally(line),
                    ));
                }
            }
            fs::write(work.join(format!("{name}.ll")), text).map_err(|e| e.to_string())?;
        }
        Ok(index_lines(index))
    })();
    let published = result.and_then(|index| {
        fs::write(work.join(INDEX), index.join("\n") + "\n").map_err(|e| e.to_string())?;
        crate::cache::publish_manifest(&work, identity)?;
        crate::cache::verify(&work, identity)?;
        if let Some(parent) = directory.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        match fs::rename(&work, directory) {
            Ok(()) => Ok(()),
            // Another process published first; its generation is complete.
            Err(_) if directory.join(INDEX).is_file() => {
                crate::cache::verify(directory, identity).map(|_| ())
            }
            Err(e) => Err(e.to_string()),
        }
    });
    let _ = fs::remove_dir_all(&work);
    published.map_err(|e| format!("building the library cache failed: {e}"))
}

/// LLVM text of one rlib's embedded bitcode, or `None` when it has none.
fn disassemble(rlib: &Path, work: &Path, tools: &Tools) -> Result<Option<String>, String> {
    fs::create_dir_all(work).map_err(|e| e.to_string())?;
    run(Command::new(&tools.ar).arg("x").arg(rlib).current_dir(work))?;
    let mut objects: Vec<_> = fs::read_dir(work)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "o"))
        .collect();
    objects.sort();
    let mut bitcode = Vec::new();
    for object in objects {
        let target = object.with_extension("bc");
        let dumped = Command::new(&tools.objcopy)
            .arg(format!("--dump-section=.llvmbc={}", target.display()))
            .arg(&object)
            .arg("/dev/null")
            .output()
            .map_err(|e| format!("objcopy: {e}"))?;
        if dumped.status.success() && fs::metadata(&target).is_ok_and(|m| m.len() > 0) {
            bitcode.push(target);
        }
    }
    if bitcode.is_empty() {
        return Ok(None);
    }
    let merged = work.join("merged.bc");
    run(Command::new(&tools.link)
        .args(&bitcode)
        .arg("-o")
        .arg(&merged))?;
    let text = work.join("merged.ll");
    run(Command::new(&tools.dis).arg(&merged).arg("-o").arg(&text))?;
    fs::read_to_string(&text)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn define_symbol(line: &str) -> Option<&str> {
    let at = line.find('@')?;
    let open = at + line[at..].find('(')?;
    Some(line[at + 1..open].trim_matches('"'))
}

fn run(command: &mut Command) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let result = command.output().map_err(|e| format!("{program}: {e}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_exports_are_ambiguous_unless_all_odr() {
        let row = |s: &str, m: &str, odr: bool| (s.to_owned(), m.to_owned(), odr, false);
        let copy = |s: &str, m: &str| (s.to_owned(), m.to_owned(), true, true);
        let lines = index_lines(vec![
            row("unique", "core", false),
            row("shared_odr", "alloc", true),
            row("shared_odr", "gimli", true),
            row("clash", "panic_abort", false),
            row("clash", "panic_unwind", false),
            row("mixed", "std", true),
            row("mixed", "test", false),
            copy("real", "aaa"),
            row("real", "zzz", true),
        ]);
        assert!(lines.contains(&"real\tzzz".to_owned()));
        assert!(lines.contains(&"unique\tcore".to_owned()));
        assert!(lines.contains(&"shared_odr\talloc".to_owned()));
        assert!(lines.contains(&format!("clash\t{AMBIGUOUS}")));
        assert!(lines.contains(&format!("mixed\t{AMBIGUOUS}")));
    }
}
