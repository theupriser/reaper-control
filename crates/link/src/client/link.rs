use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use protocol::message::{ClientMessage, ServerMessage, encode_message};

use crate::Endpoint;
use crate::wire::MessageReader;
use crate::wire::ReadError;

pub(super) const POLL: Duration = Duration::from_millis(20);

/// One open socket to the extension: send messages, poll for messages.
pub(super) struct Link {
    stream: TcpStream,
    reader: MessageReader,
}

impl Link {
    pub(super) fn open(endpoint: &Endpoint) -> Option<Self> {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(1)).ok()?;
        stream.set_nodelay(true).ok()?;
        stream.set_read_timeout(Some(POLL)).ok()?;
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .ok()?;
        let reader = MessageReader::new(stream.try_clone().ok()?);
        Some(Self { stream, reader })
    }

    pub(super) fn send(&mut self, message: &ClientMessage) -> Option<()> {
        self.stream.write_all(&encode_message(message).ok()?).ok()
    }

    pub(super) fn poll(&mut self) -> Result<Option<ServerMessage>, ReadError> {
        self.reader.poll()
    }
}
