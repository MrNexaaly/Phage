//! Library cache integrity: exact input identity plus SHA-256 of every
//! published file. Hash the same bytes the parser consumes; failures are
//! unknown, never a trusted cache. Staging ownership uses exclusive mkdir.
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
const MANIFEST: &str = "manifest.tsv";

pub fn hash(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("hash stdin missing")?
        .write_all(bytes)
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("cache SHA-256 failed".into());
    }
    String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_owned)
        .ok_or("invalid cache SHA-256 output".into())
}

fn filename(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_- .".contains(&b))
        && !name.contains(' ')
        && !name.starts_with('.')
}

pub fn read(
    directory: &Path,
    name: &str,
    hashes: &BTreeMap<String, String>,
) -> Result<Vec<u8>, String> {
    if !filename(name) {
        return Err("invalid cache filename".into());
    }
    let expected = hashes
        .get(name)
        .ok_or_else(|| format!("cache manifest lacks {name}"))?;
    let bytes =
        fs::read(directory.join(name)).map_err(|e| format!("cache integrity {name}: {e}"))?;
    if hash(&bytes)? != *expected {
        return Err(format!(
            "cache integrity mismatch: {name}; refusing cached IR"
        ));
    }
    Ok(bytes)
}

pub fn verify(directory: &Path, identity: &str) -> Result<BTreeMap<String, String>, String> {
    let text = fs::read_to_string(directory.join(MANIFEST))
        .map_err(|e| format!("cache integrity manifest: {e}"))?;
    let mut lines = text.lines();
    if lines.next() != Some("phage cache manifest 1") {
        return Err("cache manifest format mismatch".into());
    }
    let mut hashes = BTreeMap::new();
    for line in lines {
        let (name, digest) = line.split_once('\t').ok_or("invalid cache manifest row")?;
        if !filename(name)
            || digest.len() != 64
            || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            || hashes.insert(name.into(), digest.into()).is_some()
        {
            return Err("invalid cache manifest entry".into());
        }
    }
    if read(directory, "identity.txt", &hashes)? != identity.as_bytes() {
        return Err("cache input identity mismatch".into());
    }
    if !hashes.contains_key("index.tsv") {
        return Err("cache manifest lacks index".into());
    }
    for name in hashes.keys() {
        read(directory, name, &hashes)?;
    }
    Ok(hashes)
}

pub fn publish_manifest(directory: &Path, identity: &str) -> Result<(), String> {
    fs::write(directory.join("identity.txt"), identity).map_err(|e| e.to_string())?;
    let mut names = fs::read_dir(directory)
        .map_err(|e| e.to_string())?
        .map(|r| {
            r.map_err(|e| e.to_string()).and_then(|e| {
                e.file_name()
                    .into_string()
                    .map_err(|_| "cache filename is not UTF-8".into())
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    names.sort();
    let mut text = String::from("phage cache manifest 1\n");
    for name in names {
        if name == MANIFEST {
            continue;
        }
        if !filename(&name) {
            return Err("invalid cache filename".into());
        }
        text.push_str(&format!(
            "{name}\t{}\n",
            hash(&fs::read(directory.join(&name)).map_err(|e| e.to_string())?)?
        ));
    }
    fs::write(directory.join(MANIFEST), text).map_err(|e| e.to_string())
}

pub fn staging(directory: &Path) -> Result<PathBuf, String> {
    let parent = directory.parent().ok_or("cache parent missing")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for attempt in 0..100 {
        let path =
            directory.with_extension(format!("partial-{}-{stamp}-{attempt}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("exclusive cache staging exhausted".into())
}
#[cfg(test)]
mod tests;
