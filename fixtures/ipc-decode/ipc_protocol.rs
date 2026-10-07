use std::collections::HashSet;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

/// Maximum encoded environment size, including names, '=' and terminators.
pub const MAX_ENV: usize = 1024 * 1024;
/// Maximum frame payload; lengths are checked before allocation.
pub const MAX_FRAME: usize = 2 * 1024 * 1024;
const MAX_ITEMS: usize = 4096;
const MAX_TEXT: usize = 8192;

/// The client's complete request for a new window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewWindow {
    /// Command; empty selects the user's shell.
    pub argv: Vec<OsString>,
    /// Requested working directory; None uses the server's default.
    pub cwd: Option<PathBuf>,
    /// Complete child environment, preserving Unix bytes.
    pub env: Vec<(OsString, OsString)>,
    /// Keep the connection for one Exit reply following Ack.
    pub wait: bool,
    /// Optional Wayland application identifier.
    pub app_id: Option<String>,
    /// Optional window title.
    pub title: Option<String>,
    /// Optional Wayland activation token, consumed by the server.
    pub activation_token: Option<String>,
}

/// Server responses; Exit is legal only after Ack for a waiting client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// Window creation completed.
    Ack { window_id: u64 },
    /// Conventional shell exit status (0..=255).
    Exit { status: i32 },
    /// Request failed; message is at most 8192 bytes.
    Error { message: String },
}

pub(crate) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn bytes_valid(bytes: &[u8]) -> bool {
    !bytes.contains(&0)
}

impl NewWindow {
    /// Validate all fields before encoding or starting child work.
    pub fn validate(&self) -> io::Result<()> {
        if self.argv.len() > MAX_ITEMS || self.env.len() > MAX_ITEMS {
            return Err(invalid("too many arguments or environment entries"));
        }
        let mut total = 0usize;
        for arg in &self.argv {
            if !bytes_valid(arg.as_bytes()) || arg.len() >= 128 * 1024 {
                return Err(invalid("invalid or oversized argument"));
            }
            total = total.saturating_add(arg.len() + 1);
        }
        if total > MAX_ENV || self.argv.first().is_some_and(|a| a.is_empty()) {
            return Err(invalid("invalid argv"));
        }
        if self.cwd.as_ref().is_some_and(|p| {
            p.as_os_str().is_empty()
                || p.as_os_str().len() > 4096
                || !bytes_valid(p.as_os_str().as_bytes())
        }) {
            return Err(invalid("invalid cwd"));
        }
        total = 0;
        let mut keys = HashSet::new();
        for (key, value) in &self.env {
            if key.is_empty()
                || key.as_bytes().contains(&b'=')
                || !bytes_valid(key.as_bytes())
                || !bytes_valid(value.as_bytes())
                || !keys.insert(key)
            {
                return Err(invalid("invalid or duplicate environment name/value"));
            }
            total = total
                .saturating_add(key.len())
                .saturating_add(value.len())
                .saturating_add(2);
            if total > MAX_ENV {
                return Err(invalid("environment exceeds 1 MiB"));
            }
        }
        for text in [&self.app_id, &self.title, &self.activation_token]
            .into_iter()
            .flatten()
        {
            if text.len() > MAX_TEXT || !bytes_valid(text.as_bytes()) {
                return Err(invalid("invalid or oversized text field"));
            }
        }
        Ok(())
    }
}

struct Writer(Vec<u8>);
impl Writer {
    fn new(kind: u8) -> Self {
        Self(vec![b'T', b'S', b'I', b'P', 1, 0, kind])
    }
    fn u32(&mut self, n: u32) {
        self.0.extend(n.to_le_bytes());
    }
    fn bytes(&mut self, bytes: &[u8]) {
        self.u32(bytes.len() as u32);
        self.0.extend(bytes);
    }
    fn optional(&mut self, bytes: Option<&[u8]>) {
        match bytes {
            None => self.0.push(0),
            Some(b) => {
                self.0.push(1);
                self.bytes(b);
            }
        }
    }
    fn finish(self) -> io::Result<Vec<u8>> {
        if self.0.len() > MAX_FRAME {
            return Err(invalid("message exceeds 2 MiB"));
        }
        let mut framed = Vec::with_capacity(self.0.len() + 4);
        framed.extend((self.0.len() as u32).to_le_bytes());
        framed.extend(self.0);
        Ok(framed)
    }
}

pub(crate) fn encode_request(request: &NewWindow) -> io::Result<Vec<u8>> {
    request.validate()?;
    let mut w = Writer::new(1);
    w.0.push(u8::from(request.wait));
    w.u32(request.argv.len() as u32);
    for arg in &request.argv {
        w.bytes(arg.as_bytes());
    }
    w.optional(request.cwd.as_ref().map(|p| p.as_os_str().as_bytes()));
    w.u32(request.env.len() as u32);
    for (key, value) in &request.env {
        w.bytes(key.as_bytes());
        w.bytes(value.as_bytes());
    }
    for text in [&request.app_id, &request.title, &request.activation_token] {
        w.optional(text.as_ref().map(|s| s.as_bytes()));
    }
    w.finish()
}
pub(crate) fn encode_reply(reply: &Reply) -> io::Result<Vec<u8>> {
    let mut w = Writer::new(match reply {
        Reply::Ack { .. } => 2,
        Reply::Exit { .. } => 3,
        Reply::Error { .. } => 4,
    });
    match reply {
        Reply::Ack { window_id } => w.0.extend(window_id.to_le_bytes()),
        Reply::Exit { status } => {
            if !(0..=255).contains(status) {
                return Err(invalid("invalid exit status"));
            }
            w.0.extend(status.to_le_bytes());
        }
        Reply::Error { message } => {
            if message.len() > MAX_TEXT || !bytes_valid(message.as_bytes()) {
                return Err(invalid("invalid error message"));
            }
            w.bytes(message.as_bytes());
        }
    }
    w.finish()
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(invalid("truncated message"));
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }
    fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> io::Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn bytes(&mut self) -> io::Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn optional(&mut self) -> io::Result<Option<&'a [u8]>> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(self.bytes()?)),
            _ => Err(invalid("invalid optional tag")),
        }
    }
    fn count(&mut self) -> io::Result<usize> {
        let n = self.u32()? as usize;
        if n > MAX_ITEMS {
            Err(invalid("too many fields"))
        } else {
            Ok(n)
        }
    }
    fn text(&mut self) -> io::Result<Option<String>> {
        self.optional()?
            .map(|b| {
                std::str::from_utf8(b)
                    .map(str::to_owned)
                    .map_err(|_| invalid("invalid UTF-8"))
            })
            .transpose()
    }
    fn end(self) -> io::Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid("trailing message bytes"))
        }
    }
    fn header(&mut self) -> io::Result<u8> {
        if self.take(6)? != b"TSIP\x01\x00" {
            return Err(invalid("invalid protocol magic or version"));
        }
        self.byte()
    }
}
pub(crate) fn decode_request(payload: &[u8]) -> io::Result<NewWindow> {
    let mut r = Reader(payload);
    if r.header()? != 1 {
        return Err(invalid("expected NewWindow"));
    }
    let wait = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(invalid("invalid wait flag")),
    };
    let count = r.count()?;
    let mut argv = Vec::with_capacity(count);
    for _ in 0..count {
        argv.push(OsString::from_vec(r.bytes()?.to_vec()));
    }
    let cwd = r
        .optional()?
        .map(|b| PathBuf::from(OsString::from_vec(b.to_vec())));
    let count = r.count()?;
    let mut env = Vec::with_capacity(count);
    for _ in 0..count {
        env.push((
            OsString::from_vec(r.bytes()?.to_vec()),
            OsString::from_vec(r.bytes()?.to_vec()),
        ));
    }
    let request = NewWindow {
        argv,
        cwd,
        env,
        wait,
        app_id: r.text()?,
        title: r.text()?,
        activation_token: r.text()?,
    };
    r.end()?;
    request.validate()?;
    Ok(request)
}
pub(crate) fn decode_reply(payload: &[u8]) -> io::Result<Reply> {
    let mut r = Reader(payload);
    let reply = match r.header()? {
        2 => {
            let b = r.take(8)?;
            Reply::Ack {
                window_id: u64::from_le_bytes(b.try_into().map_err(|_| invalid("invalid id"))?),
            }
        }
        3 => {
            let status = r.u32()?;
            if status > 255 {
                return Err(invalid("invalid exit status"));
            }
            Reply::Exit {
                status: status as i32,
            }
        }
        4 => {
            let b = r.bytes()?;
            if b.len() > MAX_TEXT || !bytes_valid(b) {
                return Err(invalid("invalid error message"));
            }
            Reply::Error {
                message: std::str::from_utf8(b)
                    .map_err(|_| invalid("invalid UTF-8"))?
                    .to_owned(),
            }
        }
        _ => return Err(invalid("unexpected server message")),
    };
    r.end()?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_truncation_is_rejected() {
        let request = NewWindow {
            argv: vec!["sh".into()],
            cwd: Some("/tmp".into()),
            env: vec![("A".into(), "B".into())],
            wait: true,
            app_id: Some("id".into()),
            title: None,
            activation_token: None,
        };
        let encoded = encode_request(&request).unwrap();
        for end in 0..encoded.len() - 4 {
            assert!(decode_request(&encoded[4..4 + end]).is_err());
        }
        assert_eq!(decode_request(&encoded[4..]).unwrap(), request);
        for reply in [
            Reply::Ack { window_id: 99 },
            Reply::Exit { status: 130 },
            Reply::Error {
                message: "bad".into(),
            },
        ] {
            let bytes = encode_reply(&reply).unwrap();
            for end in 0..bytes.len() - 4 {
                assert!(decode_reply(&bytes[4..4 + end]).is_err());
            }
            assert_eq!(decode_reply(&bytes[4..]).unwrap(), reply);
        }
    }
    #[test]
    fn unknown_tags_version_counts_and_trailing_bytes() {
        let request = NewWindow {
            argv: Vec::new(),
            cwd: None,
            env: Vec::new(),
            wait: false,
            app_id: None,
            title: None,
            activation_token: None,
        };
        let mut bytes = encode_request(&request).unwrap()[4..].to_vec();
        assert!(decode_request(&bytes).is_ok());
        bytes.push(0);
        assert!(decode_request(&bytes).is_err());
        bytes.pop();
        for (offset, value) in [(4, 2), (6, 9), (7, 2), (12, 2), (8, 255), (9, 255)] {
            let mut mutated = bytes.clone();
            mutated[offset] = value;
            assert!(decode_request(&mutated).is_err());
        }
    }
}

#[cfg(test)]
mod extra_validation_tests {
    use super::*;
    #[test]
    fn oversized_environment_is_rejected_by_the_server_decoder() {
        let mut writer = Writer::new(1);
        writer.0.push(0);
        writer.u32(0);
        writer.optional(None);
        writer.u32(1);
        writer.bytes(b"HUGE");
        writer.bytes(&vec![b'x'; MAX_ENV]);
        for _ in 0..3 {
            writer.optional(None);
        }
        let framed = writer.finish().unwrap();
        assert!(
            decode_request(&framed[4..])
                .unwrap_err()
                .to_string()
                .contains("1 MiB")
        );
    }
}
