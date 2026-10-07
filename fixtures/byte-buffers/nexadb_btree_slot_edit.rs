//! PENDING on Phage: `unsupported entry argument: ptr %bytes` (zero explored paths, 2026-10-05); move back to verify/nexadb-btree/ once Phage models slice pointers.
//! Check the actual cached/full views and private byte splicer against arbitrary bounded page bytes.
//! phage-args: --unwind 4096 --states 10000 --timeout 100 --maxMemoryMiB 2048
extern crate self as nexadb_page;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-page/src/lib.rs"]
mod page;
pub use page::*;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/lib.rs"]
mod tree;
pub use tree::{
    validate_node, PageSink, PageSource, Root, TreeError, TreeStats, MAX_KEY_LEN, MAX_VALUE_LEN,
};
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/edit.rs"]
mod edit;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/node.rs"]
mod node;
#[path = "/home/zero/Dev/Nexaaly/NexaDB/crates/nexadb-btree/src/view.rs"]
mod view;
#[unsafe(no_mangle)]
pub fn phage_target(bytes: &[u8; PAGE_SIZE], index: u16, remove: u16, count: u64) -> bool {
    let mut page = Box::new(*bytes);
    let _ = validate_node(&page);
    if let Ok((kind, length)) = view::header(&page) {
        if let Ok(v) = view::View::cached(&body(&page)[..length], kind) {
            let _ = v.item(usize::from(index));
            let _ = v.key(usize::from(index));
            let _ = v.lower(b"key", false);
            let _ = v.slot(b"key");
        }
    }
    let _ = edit::splice(
        &mut page,
        usize::from(index),
        usize::from(remove),
        &[b"\x01\x00\x01\x00kv"],
        count,
    );
    true
}
