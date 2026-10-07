//! PENDING on Phage: `unsupported ptr %bytes` (zero explored paths, 2026-10-05); move back to verify/nexadb-btree/ once Phage models slice pointers.
//! Prove bounds-safe construction, binary search and selected entries of the actual slot parser.
//! phage-args: --unwind 4096 --states 10000 --timeout 1000 --maxMemoryMiB 2048
extern crate self as nexadb_page;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-page/src/lib.rs"]
mod page;
pub use page::*;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/lib.rs"]
mod tree;
pub use tree::{MAX_KEY_LEN, MAX_VALUE_LEN, PageSink, PageSource, Root, TreeError, TreeStats};
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/node.rs"]
mod node;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/view.rs"]
mod view;
#[unsafe(no_mangle)]
pub fn phage_target(bytes: &[u8; 14], length: u16, branch: bool, index: u16) -> bool {
    let Some(body) = bytes.get(..usize::from(length)) else { return true; };
    if let Ok(v) = view::View::new(body, if branch { PageKind::Branch } else { PageKind::Leaf }) {
        let _ = v.item(usize::from(index));
        let _ = v.key(usize::from(index));
        let _ = v.lower(body, false);
        let _ = v.lower(body, true);
        let _ = v.slot(body);
    }
    true
}
// Scratch-only linkage adapter for current NexaDB's crate-root imports.
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/edit.rs"]
mod edit;
pub use tree::validate_node;
