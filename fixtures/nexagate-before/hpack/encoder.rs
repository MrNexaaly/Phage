//! Fields to header blocks (RFC 7541 section 6), for our responses.
//!
//! Policy:
//! - an exact static or dynamic match is sent as a one-byte index;
//! - otherwise the field is a literal that reuses an indexed name, and is
//!   added to the dynamic table so repeats (the site's long security headers,
//!   `content-type`, `vary`) cost a byte or two from then on;
//! - values that change per response (`date`, `content-length`, `etag`, ...)
//!   are not indexed, since they would only churn the table;
//! - credentials (`set-cookie`, `authorization`, ...) are "never indexed"
//!   (section 7.1.3), which also tells intermediaries not to index them.
//!
//! Names are lowercased, as HTTP/2 requires (RFC 9113 section 8.2).

use super::{
    encode_int, encode_str,
    table::{DynamicTable, ENTRY_OVERHEAD, find_static},
    tables::STATIC,
};

/// The table size we use, even if the peer allows more.
const TABLE_SIZE: usize = 4096;

const VOLATILE: &[&[u8]] = &[
    b"content-length",
    b"date",
    b"etag",
    b"last-modified",
    b"content-range",
    b"age",
    b"expires",
    b"location",
];

const SENSITIVE: &[&[u8]] = &[
    b"set-cookie",
    b"authorization",
    b"proxy-authorization",
    b"cookie",
];

#[derive(Debug)]
pub struct Encoder {
    table: DynamicTable,
    /// Table size changes not yet signalled: the smallest, then the final.
    update: Option<(usize, usize)>,
    name: Vec<u8>,
}

impl Default for Encoder {
    fn default() -> Self {
        Encoder::new()
    }
}

impl Encoder {
    /// The peer's decoder starts at 4096 bytes (RFC 7541 section 4.2).
    pub fn new() -> Encoder {
        Encoder {
            table: DynamicTable::new(TABLE_SIZE),
            update: None,
            name: Vec::new(),
        }
    }

    /// Applies the peer's SETTINGS_HEADER_TABLE_SIZE.
    pub fn set_peer_max(&mut self, max: usize) {
        let size = max.min(TABLE_SIZE);
        if size == self.table.max() && self.update.is_none() {
            return;
        }
        self.table.set_max(size);
        self.update = Some(match self.update {
            Some((smallest, _)) => (smallest.min(size), size),
            None => (size, size),
        });
    }

    /// Encodes one complete header block.
    pub fn encode<'f>(
        &mut self,
        fields: impl Iterator<Item = (&'f [u8], &'f [u8])>,
        out: &mut Vec<u8>,
    ) {
        if let Some((smallest, last)) = self.update.take() {
            if smallest < last {
                encode_int(out, 0x20, 5, smallest as u64);
            }
            encode_int(out, 0x20, 5, last as u64);
        }
        for (name, value) in fields {
            self.field(name, value, out);
        }
    }

    fn field(&mut self, name: &[u8], value: &[u8], out: &mut Vec<u8>) {
        self.name.clear();
        self.name.extend(name.iter().map(u8::to_ascii_lowercase));
        let name = &self.name;
        let sensitive = SENSITIVE.contains(&name.as_slice());
        let (exact, static_name) = find_static(name, value);
        if !sensitive {
            if let Some(index) = exact {
                encode_int(out, 0x80, 7, index as u64);
                return;
            }
            if let Some(i) = self
                .table
                .iter()
                .position(|(n, v)| n == name.as_slice() && v == value)
            {
                encode_int(out, 0x80, 7, (STATIC.len() + 1 + i) as u64);
                return;
            }
        }
        let name_index = static_name.or_else(|| {
            self.table
                .iter()
                .position(|(n, _)| n == name.as_slice())
                .map(|i| STATIC.len() + 1 + i)
        });
        let index_it = !sensitive
            && !VOLATILE.contains(&name.as_slice())
            && name.len() + value.len() + ENTRY_OVERHEAD <= self.table.max() / 2;
        let (flags, prefix) = match (index_it, sensitive) {
            (true, _) => (0x40, 6),
            (false, true) => (0x10, 4),
            (false, false) => (0x00, 4),
        };
        match name_index {
            Some(index) => encode_int(out, flags, prefix, index as u64),
            None => {
                encode_int(out, flags, prefix, 0);
                encode_str(out, 0, 7, name);
            }
        }
        encode_str(out, 0, 7, value);
        if index_it {
            let name = std::mem::take(&mut self.name);
            self.table.insert(&name, value);
            self.name = name;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Decoder, FieldList};
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        let digits: Vec<u8> = text.bytes().filter(|b| b.is_ascii_hexdigit()).collect();
        digits
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }

    fn encode(e: &mut Encoder, fields: &[(&str, &str)]) -> Vec<u8> {
        let mut out = Vec::new();
        e.encode(
            fields.iter().map(|(n, v)| (n.as_bytes(), v.as_bytes())),
            &mut out,
        );
        out
    }

    #[test]
    fn matches_rfc_7541_c4_byte_for_byte() {
        let mut e = Encoder::new();
        assert_eq!(
            encode(
                &mut e,
                &[
                    (":method", "GET"),
                    (":scheme", "http"),
                    (":path", "/"),
                    (":authority", "www.example.com")
                ]
            ),
            hex("8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff")
        );
        assert_eq!(
            encode(
                &mut e,
                &[
                    (":method", "GET"),
                    (":scheme", "http"),
                    (":path", "/"),
                    (":authority", "www.example.com"),
                    ("cache-control", "no-cache")
                ]
            ),
            hex("8286 84be 5886 a8eb 1064 9cbf")
        );
        assert_eq!(
            encode(
                &mut e,
                &[
                    (":method", "GET"),
                    (":scheme", "https"),
                    (":path", "/index.html"),
                    (":authority", "www.example.com"),
                    ("custom-key", "custom-value")
                ]
            ),
            hex("8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf")
        );
    }

    #[test]
    fn repeated_headers_shrink_and_round_trip() {
        let csp =
            "default-src 'self'; script-src 'self' https://js.stripe.com; frame-ancestors 'none'";
        let response = [
            (":status", "200"),
            ("Content-Type", "text/html; charset=utf-8"),
            ("Content-Security-Policy", csp),
            ("Date", "Mon, 21 Oct 2013 20:13:21 GMT"),
            ("Set-Cookie", "session=secret"),
        ];
        let mut e = Encoder::new();
        let mut d = Decoder::new(4096);
        let first = encode(&mut e, &response);
        let second = encode(&mut e, &response);
        assert!(
            second.len() < first.len() / 2,
            "{} then {} bytes",
            first.len(),
            second.len()
        );
        let mut out = FieldList::default();
        for block in [first, second] {
            d.decode(&block, 1 << 20, &mut out).unwrap();
            let decoded: Vec<(&[u8], &[u8])> = out.iter().collect();
            assert_eq!(
                decoded[1],
                (&b"content-type"[..], &b"text/html; charset=utf-8"[..]),
                "names lowercased"
            );
            assert_eq!(decoded[2].1, csp.as_bytes());
            assert_eq!(decoded[4], (&b"set-cookie"[..], &b"session=secret"[..]));
        }
        assert!(
            !d.table()
                .iter()
                .any(|(n, _)| n == b"set-cookie" || n == b"date"),
            "credentials and volatile values stay out of the table"
        );
    }

    #[test]
    fn signals_table_size_changes() {
        let mut e = Encoder::new();
        e.set_peer_max(0);
        e.set_peer_max(1024);
        let block = encode(&mut e, &[(":status", "200")]);
        // Smallest (0) then final (1024), then the indexed field.
        assert_eq!(block, [0x20, 0x3f, 0xe1, 0x07, 0x88]);
        let mut d = Decoder::new(4096);
        let mut out = FieldList::default();
        d.decode(&block, 1 << 20, &mut out).unwrap();
        assert_eq!(d.table().max(), 1024);
    }
}
