use serde::de::DeserializeOwned;
use std::io::{self, BufRead, Write};

pub mod proto_json;
pub mod proto_postcard;

pub trait Message: serde::Serialize + DeserializeOwned {
    fn receive<R: BufRead>(reader: &mut R) -> io::Result<Self>;
    fn send<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

#[macro_export]
macro_rules! impl_message {
    ($type_name:ident) => {
        impl Message for $type_name {
            fn receive<R: BufRead>(reader: &mut R) -> io::Result<Self> {
                Self::receive_proto(reader)
            }

            fn send<W: Write>(&self, writer: &mut W) -> io::Result<()> {
                self.send_proto(writer)
            }
        }
    };
}

pub fn send_msg<M: Message, W: Write>(msg: &M, writer: &mut W) -> io::Result<()> {
    msg.send(writer)
}

pub fn receive_msg<M: Message, R: BufRead>(reader: &mut R) -> io::Result<M> {
    M::receive(reader)
}
