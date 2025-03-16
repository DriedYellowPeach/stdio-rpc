use colored::Colorize;
use serde::de::DeserializeOwned;
use std::io::{self, BufRead, Read, Write};

use crate::{Message, impl_message};

const FMT_TOTAL_WIDTH: usize = 36;
const FMT_PADDING: usize = 4;

fn read_u64_be<R: Read>(reader: &mut R) -> io::Result<usize> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(usize::from_be_bytes(buf))
}

fn write_u64_be<W: Write>(writer: &mut W, value: usize) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes()) // Convert and write as Big-Endian
}

trait ProtoPostcard: serde::Serialize + DeserializeOwned {
    fn receive_proto<R: BufRead>(reader: &mut R) -> io::Result<Self> {
        let msg_len = read_u64_be(reader)? as usize;
        let mut buf = vec![0u8; msg_len];

        reader.read_exact(&mut buf)?;
        postcard::from_bytes::<Self>(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn send_proto<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = postcard::to_allocvec(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        write_u64_be(writer, bytes.len())?;
        writer.write_all(&bytes)?;
        writer.flush()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum C2SMsg {
    Request(String),
    Reply(i64),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum S2CMsg {
    Response(i64),
    Query(char),
    Log(String),
    BadSeq,
}

impl std::fmt::Display for C2SMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 13;

        match self {
            C2SMsg::Request(content) => {
                let truncated_content = if content.len() > content_width {
                    // Truncate the string if it's too long
                    format!("{}...", &content[..content_width - 3])
                } else {
                    content.clone()
                };

                write!(
                    f,
                    "    |----{}: {:>width$}---▶|",
                    "Req".bold().green(),
                    truncated_content.green(),
                    width = content_width
                )
            }
            C2SMsg::Reply(val) => {
                write!(
                    f,
                    "    |----{}: {:>width$}!---▶|",
                    "Reply".bold().bright_yellow().italic(),
                    val.to_string().yellow(),
                    width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 16
                )
            }
        }
    }
}

impl std::fmt::Display for S2CMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2CMsg::Response(val) => {
                write!(
                    f,
                    "    |◀---{}: {:>width$}----|",
                    "Response".bold().bright_green().italic(),
                    val.to_string().green(),
                    width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 18
                )
            }
            S2CMsg::Query(val) => {
                write!(
                    f,
                    "    |◀---{}: {:>width$}?----|",
                    "Query".bold().yellow(),
                    val.to_string().yellow(),
                    width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 16
                )
            }
            S2CMsg::Log(s) => {
                write!(
                    f,
                    "    |◀---Log: {:>width$}!---|",
                    s,
                    width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 12
                )
            }
            S2CMsg::BadSeq => {
                write!(
                    f,
                    "    ◀---BadSeq: {:>width$}!----",
                    "Bad seq",
                    width = FMT_TOTAL_WIDTH - 2 * FMT_PADDING - 16
                )
            }
        }
    }
}

impl ProtoPostcard for C2SMsg {}
impl ProtoPostcard for S2CMsg {}

impl_message!(C2SMsg);
impl_message!(S2CMsg);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_send_msg() {
        let mut buf = Vec::new();
        let messages = ["hello", "world"];

        for &msg in &messages {
            C2SMsg::Request(msg.to_string()).send(&mut buf).unwrap();
        }

        let mut cursor = Cursor::new(buf);

        for &expected in &messages {
            let received = C2SMsg::receive(&mut cursor).unwrap();
            assert!(matches!(received, C2SMsg::Request(ref s) if s == expected));
        }
    }
}
