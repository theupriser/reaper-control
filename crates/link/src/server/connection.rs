use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

use super::hub::Frame;
use super::shared::Shared;
use crate::wire::{MessageReader, ReadError};

mod admitted;
mod client_writer;
mod command_session;
mod handshake;
mod recent_answers;

use client_writer::ClientWriter;
use command_session::CommandSession;
use handshake::Handshake;

const POLL: Duration = Duration::from_millis(20);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);
const OUTBOX: usize = 64;

/// One client from accept to close: handshake, register, serve, clean up.
pub(super) struct Connection<'a> {
    pub(super) stream: TcpStream,
    pub(super) shared: &'a Shared,
}

impl Connection<'_> {
    pub(super) fn run(self) -> Result<(), ReadError> {
        let stream = &self.stream;
        stream.set_nonblocking(false)?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(POLL))?;
        stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
        let mut reader = MessageReader::new(stream.try_clone()?);

        let handshake = Handshake {
            token: &self.shared.token,
            stop: &self.shared.stop,
        };
        let Some(admitted) = handshake.admit(&mut reader)? else {
            return Ok(());
        };

        let (outbox, inbox) = sync_channel::<Frame>(OUTBOX);
        let writer = ClientWriter::spawn(stream.try_clone()?, inbox)?;
        let id = self.shared.hub().register(
            &self.shared.extension_version,
            admitted.resume_from,
            stream,
            &outbox,
        )?;
        let result = CommandSession {
            shared: self.shared,
            outbox: &outbox,
        }
        .run(&mut reader);
        self.shared.hub().remove(id);
        let _ = stream.shutdown(Shutdown::Both);
        drop(outbox);
        writer.join();
        result
    }
}
