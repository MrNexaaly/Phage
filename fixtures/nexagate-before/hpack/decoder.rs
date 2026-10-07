//! Header blocks to fields (RFC 7541 section 6).
//!
//! Dynamic table size updates are accepted only at the start of a block and
//! never above the size we advertised (section 4.2). The list size (names +
//! values + 32 per field, RFC 9113 section 6.5.2) is bounded; past the bound
//! the block is still decoded, so the dynamic table stays in step with the
//! peer's, and the caller answers that one stream with 431.

use std::ops::Range;

use super::{
    HpackError, decode_int, decode_str,
    table::{DynamicTable, ENTRY_OVERHEAD, lookup},
};

/// Decoded fields in one reusable buffer.
#[derive(Debug, Default)]
pub struct FieldList {
    data: Vec<u8>,
    fields: Vec<(Range<usize>, Range<usize>)>,
}

impl FieldList {
    pub fn clear(&mut self) {
        self.data.clear();
        self.fields.clear();
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> + Clone {
        self.fields
            .iter()
            .map(|(n, v)| (&self.data[n.clone()], &self.data[v.clone()]))
    }

    pub fn push(&mut self, name: &[u8], value: &[u8]) {
        let n = self.data.len()..self.data.len() + name.len();
        self.data.extend_from_slice(name);
        let v = self.data.len()..self.data.len() + value.len();
        self.data.extend_from_slice(value);
        self.fields.push((n, v));
    }
}

#[derive(Debug)]
pub struct Decoder {
    table: DynamicTable,
    /// The SETTINGS_HEADER_TABLE_SIZE we advertised: the ceiling for updates.
    max_allowed: usize,
}

impl Decoder {
    pub fn new(max_allowed: usize) -> Decoder {
        Decoder {
            table: DynamicTable::new(max_allowed),
            max_allowed,
        }
    }

    pub fn table(&self) -> &DynamicTable {
        &self.table
    }

    /// Decodes one complete header block into `out` (cleared first).
    pub fn decode(
        &mut self,
        block: &[u8],
        max_list: usize,
        out: &mut FieldList,
    ) -> Result<(), HpackError> {
        out.clear();
        let mut pos = 0;
        let mut list_size = 0usize;
        let mut too_large = false;
        let mut fields_started = false;
        while pos < block.len() {
            let byte = block[pos];
            let mark = out.data.len();
            // (name range, value range, insert into the table)
            let (name, value, index_it) = if byte & 0x80 != 0 {
                // Indexed field (section 6.1).
                let (index, used) = decode_int(&block[pos..], 7)?;
                pos += used;
                let (n, v) = lookup(&self.table, index as usize)
                    .ok_or(HpackError::Malformed("index out of range"))?;
                let name = push(&mut out.data, n);
                let value = push(&mut out.data, v);
                (name, value, false)
            } else if byte & 0xe0 == 0x20 {
                // Dynamic table size update (section 6.3).
                if fields_started {
                    return Err(HpackError::Malformed("table size update after a field"));
                }
                let (size, used) = decode_int(&block[pos..], 5)?;
                pos += used;
                if size as usize > self.max_allowed {
                    return Err(HpackError::Malformed(
                        "table size above the advertised limit",
                    ));
                }
                self.table.set_max(size as usize);
                continue;
            } else {
                // Literal (sections 6.2.1-6.2.3): with incremental indexing
                // (01), without indexing (0000) or never indexed (0001).
                let (prefix, index_it) = if byte & 0xc0 == 0x40 {
                    (6, true)
                } else {
                    (4, false)
                };
                let (index, used) = decode_int(&block[pos..], prefix)?;
                pos += used;
                let name = if index == 0 {
                    let start = out.data.len();
                    pos += decode_str(&block[pos..], 7, &mut out.data)?;
                    start..out.data.len()
                } else {
                    let (n, _) = lookup(&self.table, index as usize)
                        .ok_or(HpackError::Malformed("index out of range"))?;
                    push(&mut out.data, n)
                };
                let start = out.data.len();
                pos += decode_str(&block[pos..], 7, &mut out.data)?;
                (name, start..out.data.len(), index_it)
            };
            fields_started = true;
            if index_it {
                self.table
                    .insert(&out.data[name.clone()], &out.data[value.clone()]);
            }
            list_size += name.len() + value.len() + ENTRY_OVERHEAD;
            if list_size > max_list {
                too_large = true;
            }
            if too_large {
                out.data.truncate(mark); // keep decoding, stop storing
            } else {
                out.fields.push((name, value));
            }
        }
        if too_large {
            out.clear();
            return Err(HpackError::TooLarge);
        }
        Ok(())
    }
}

fn push(data: &mut Vec<u8>, bytes: &[u8]) -> Range<usize> {
    let start = data.len();
    data.extend_from_slice(bytes);
    start..data.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        let digits: Vec<u8> = text.bytes().filter(|b| b.is_ascii_hexdigit()).collect();
        digits
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }

    fn decode(decoder: &mut Decoder, block: &str) -> Vec<(String, String)> {
        let mut out = FieldList::default();
        decoder.decode(&hex(block), 1 << 20, &mut out).unwrap();
        out.iter()
            .map(|(n, v)| {
                (
                    String::from_utf8(n.to_vec()).unwrap(),
                    String::from_utf8(v.to_vec()).unwrap(),
                )
            })
            .collect()
    }

    fn fields(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn rfc_7541_c3_requests_without_huffman() {
        let mut d = Decoder::new(4096);
        assert_eq!(
            decode(&mut d, "8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d"),
            fields(&[
                (":method", "GET"),
                (":scheme", "http"),
                (":path", "/"),
                (":authority", "www.example.com")
            ])
        );
        assert_eq!(d.table().size(), 57);
        assert_eq!(
            decode(&mut d, "8286 84be 5808 6e6f 2d63 6163 6865"),
            fields(&[
                (":method", "GET"),
                (":scheme", "http"),
                (":path", "/"),
                (":authority", "www.example.com"),
                ("cache-control", "no-cache")
            ])
        );
        assert_eq!(
            decode(
                &mut d,
                "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65"
            ),
            fields(&[
                (":method", "GET"),
                (":scheme", "https"),
                (":path", "/index.html"),
                (":authority", "www.example.com"),
                ("custom-key", "custom-value")
            ])
        );
        assert_eq!(d.table().size(), 164);
    }

    #[test]
    fn rfc_7541_c6_responses_with_huffman_and_eviction() {
        let mut d = Decoder::new(256);
        assert_eq!(
            decode(
                &mut d,
                "4882 6402 5885 aec3 771a 4b61 96d0 7abe 9410 54d4 44a8 2005 9504 0b81 66e0 82a6 2d1b ff6e 919d 29ad 1718 63c7 8f0b 97c8 e9ae 82ae 43d3"
            ),
            fields(&[
                (":status", "302"),
                ("cache-control", "private"),
                ("date", "Mon, 21 Oct 2013 20:13:21 GMT"),
                ("location", "https://www.example.com")
            ])
        );
        assert_eq!(d.table().size(), 222);
        assert_eq!(
            decode(&mut d, "4883 640e ffc1 c0bf"),
            fields(&[
                (":status", "307"),
                ("cache-control", "private"),
                ("date", "Mon, 21 Oct 2013 20:13:21 GMT"),
                ("location", "https://www.example.com")
            ])
        );
        assert_eq!(d.table().size(), 222);
        assert_eq!(
            decode(
                &mut d,
                "88c1 6196 d07a be94 1054 d444 a820 0595 040b 8166 e084 a62d 1bff c05a 839b d9ab 77ad 94e7 821d d7f2 e6c7 b335 dfdf cd5b 3960 d5af 2708 7f36 72c1 ab27 0fb5 291f 9587 3160 65c0 03ed 4ee5 b106 3d50 07"
            ),
            fields(&[
                (":status", "200"),
                ("cache-control", "private"),
                ("date", "Mon, 21 Oct 2013 20:13:22 GMT"),
                ("location", "https://www.example.com"),
                ("content-encoding", "gzip"),
                (
                    "set-cookie",
                    "foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1"
                )
            ])
        );
        assert_eq!(d.table().size(), 215);
    }

    #[test]
    fn size_updates_only_first_and_within_the_limit() {
        let mut d = Decoder::new(4096);
        let mut out = FieldList::default();
        assert!(
            d.decode(&[0x3f, 0xe1, 0x1f, 0x82], 1 << 20, &mut out)
                .is_ok(),
            "update to 4096, then a field"
        );
        assert_eq!(
            d.decode(&[0x82, 0x20], 1 << 20, &mut out),
            Err(HpackError::Malformed("table size update after a field"))
        );
        let mut small = Decoder::new(100);
        assert!(
            small.decode(&[0x3f, 0x46], 1 << 20, &mut out).is_err(),
            "above the advertised 100"
        );
    }

    #[test]
    fn oversized_lists_still_update_the_table() {
        let mut d = Decoder::new(4096);
        let mut out = FieldList::default();
        // Literal with indexing: custom-key: custom-header (RFC 7541 C.2.1).
        let block = hex("400a 6375 7374 6f6d 2d6b 6579 0d63 7573 746f 6d2d 6865 6164 6572");
        assert_eq!(d.decode(&block, 10, &mut out), Err(HpackError::TooLarge));
        assert!(out.is_empty());
        assert_eq!(d.table().len(), 1, "the table entry was still added");
        assert!(
            d.decode(&[0x80], 1 << 20, &mut out).is_err(),
            "index 0 is invalid"
        );
    }
}
