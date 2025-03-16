use serde::de::DeserializeOwned;
use std::io::{self, BufRead, Write};

use crate::{Message, impl_message};

trait ProtoJson: serde::Serialize + DeserializeOwned {
    fn receive_proto<R: BufRead>(reader: &mut R) -> io::Result<Self> {
        let mut buf = String::new();
        loop {
            buf.clear();

            reader.read_line(&mut buf)?;
            buf.pop(); // Remove trailing '\n'

            if buf.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "empty line"));
            }

            // Some ill behaved macro try to use stdout for debugging
            // We ignore it here
            if !buf.starts_with('{') {
                tracing::error!("proc-macro tried to print : {}", buf);
                continue;
            }

            return serde_json::from_str(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        }
    }

    fn send_proto<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let msg = serde_json::to_string(&self)?;
        writer.write_all(msg.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()
    }
}

impl ProtoJson for Request {}
impl ProtoJson for Response {}

impl_message!(Request);
impl_message!(Response);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Request {
    ListMacros { dylib_path: String },
    ExpandMacro(u8),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Response {}
