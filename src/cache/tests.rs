//! Real hashing controls: altered/missing index, module, identity, manifest.
use super::*;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let parent = std::env::temp_dir().join("phage-cache-controls");
        let directory = staging(&parent.join("generation")).unwrap();
        fs::write(directory.join("index.tsv"), "target\tcore\n").unwrap();
        fs::write(
            directory.join("core.ll"),
            "define i1 @target() { ret i1 true }\n",
        )
        .unwrap();
        publish_manifest(&directory, "compiler identity A").unwrap();
        Self(directory)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn tampering_with_any_trusted_input_is_rejected() {
    for name in ["core.ll", "index.tsv", "identity.txt", "manifest.tsv"] {
        let f = Fixture::new();
        assert!(verify(&f.0, "compiler identity A").is_ok());
        fs::write(f.0.join(name), "altered").unwrap();
        assert!(verify(&f.0, "compiler identity A").is_err(), "{name}");
    }
    let f = Fixture::new();
    assert!(verify(&f.0, "compiler identity B").is_err());
    fs::remove_file(f.0.join("core.ll")).unwrap();
    assert!(verify(&f.0, "compiler identity A").is_err());
}
#[test]
fn changes_after_open_are_rechecked_on_module_read() {
    let f = Fixture::new();
    let hashes = verify(&f.0, "compiler identity A").unwrap();
    fs::write(f.0.join("core.ll"), "altered after open").unwrap();
    assert!(read(&f.0, "core.ll", &hashes).is_err());
}
#[test]
fn staging_directories_are_unique_and_exclusively_owned() {
    let f = Fixture::new();
    let a = staging(&f.0.join("cache")).unwrap();
    let b = staging(&f.0.join("cache")).unwrap();
    assert_ne!(a, b);
    assert!(fs::create_dir(a).is_err());
    assert!(fs::create_dir(b).is_err());
}
