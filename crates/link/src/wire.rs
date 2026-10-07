//! Reading messages off a socket.

mod message_reader;
mod read_error;

pub(crate) use message_reader::MessageReader;
pub(crate) use read_error::ReadError;
