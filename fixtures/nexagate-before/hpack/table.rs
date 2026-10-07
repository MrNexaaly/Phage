//! HPACK's index space (RFC 7541 section 2.3): the 61-entry static table,
//! then the dynamic table, newest entry first.
//!
//! The dynamic table is bounded by size, not count: an entry costs its name
//! and value lengths plus 32 bytes (section 4.1). Inserting evicts the oldest
//! entries until the new one fits; one larger than the whole table empties it
//! and is not stored (section 4.4).

use std::{collections::HashMap, collections::VecDeque, sync::OnceLock};

use super::tables::STATIC;

pub const ENTRY_OVERHEAD: usize = 32;

#[derive(Debug, Default)]
pub struct DynamicTable {
    /// Newest first.
    entries: VecDeque<(Vec<u8>, Vec<u8>)>,
    size: usize,
    max: usize,
}

impl DynamicTable {
    pub fn new(max: usize) -> DynamicTable {
        DynamicTable {
            max,
            ..DynamicTable::default()
        }
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = max;
        self.evict(0);
    }

    pub fn insert(&mut self, name: &[u8], value: &[u8]) {
        let cost = name.len() + value.len() + ENTRY_OVERHEAD;
        if cost > self.max {
            self.entries.clear();
            self.size = 0;
            return;
        }
        self.evict(cost);
        self.entries.push_front((name.to_vec(), value.to_vec()));
        self.size += cost;
    }

    /// Entry `index` counting from the newest (0).
    pub fn get(&self, index: usize) -> Option<(&[u8], &[u8])> {
        self.entries
            .get(index)
            .map(|(n, v)| (n.as_slice(), v.as_slice()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.entries
            .iter()
            .map(|(n, v)| (n.as_slice(), v.as_slice()))
    }

    fn evict(&mut self, room: usize) {
        while self.size + room > self.max {
            let Some((name, value)) = self.entries.pop_back() else {
                break;
            };
            self.size -= name.len() + value.len() + ENTRY_OVERHEAD;
        }
    }
}

/// Looks up the combined index space (1-based; static first).
pub fn lookup(table: &DynamicTable, index: usize) -> Option<(&[u8], &[u8])> {
    match index {
        0 => None,
        i if i <= STATIC.len() => {
            let (n, v) = STATIC[i - 1];
            Some((n.as_bytes(), v.as_bytes()))
        }
        i => table.get(i - STATIC.len() - 1),
    }
}

/// Per name: the first index with that name, and every `(value, index)`.
type StaticIndex = HashMap<&'static [u8], (usize, Vec<(&'static [u8], usize)>)>;

fn static_index() -> &'static StaticIndex {
    static INDEX: OnceLock<StaticIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = StaticIndex::new();
        for (i, (name, value)) in STATIC.iter().enumerate() {
            let entry = map.entry(name.as_bytes()).or_insert((i + 1, Vec::new()));
            entry.1.push((value.as_bytes(), i + 1));
        }
        map
    })
}

/// The best static match for a (lowercase) field: an exact entry, or at least
/// an entry with the same name.
pub fn find_static(name: &[u8], value: &[u8]) -> (Option<usize>, Option<usize>) {
    match static_index().get(name) {
        Some((first, values)) => (
            values.iter().find(|(v, _)| *v == value).map(|(_, i)| *i),
            Some(*first),
        ),
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_oldest_by_size() {
        let mut table = DynamicTable::new(100);
        table.insert(b"a", b"1"); // 34
        table.insert(b"b", b"2"); // 68
        table.insert(b"c", b"3"); // would be 102: "a" goes
        assert_eq!(table.len(), 2);
        assert_eq!(table.get(0), Some((&b"c"[..], &b"3"[..])));
        assert_eq!(table.get(1), Some((&b"b"[..], &b"2"[..])));
        assert_eq!(table.size(), 68);
        table.insert(&[b'x'; 80], b""); // bigger than the table: empties it
        assert!(table.is_empty() && table.size() == 0);
        table.insert(b"a", b"1");
        table.set_max(0);
        assert!(table.is_empty());
    }

    #[test]
    fn combined_index_space() {
        let mut table = DynamicTable::new(4096);
        table.insert(b"custom-key", b"custom-header");
        assert_eq!(lookup(&table, 2), Some((&b":method"[..], &b"GET"[..])));
        assert_eq!(
            lookup(&table, 62),
            Some((&b"custom-key"[..], &b"custom-header"[..]))
        );
        assert_eq!(lookup(&table, 63), None);
        assert_eq!(lookup(&table, 0), None);
        assert_eq!(find_static(b":status", b"404"), (Some(13), Some(8)));
        assert_eq!(find_static(b"content-type", b"text/html"), (None, Some(31)));
        assert_eq!(find_static(b"x-custom", b""), (None, None));
    }
}
