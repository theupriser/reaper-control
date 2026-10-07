use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::SyncSender;

use protocol::message::{ClientMessage, Outcome, ServerMessage, encode_message};

use crate::server::hub::Frame;
use crate::server::shared::Shared;
use crate::wire::{MessageReader, ReadError};

/// Answers one client's commands and pings until it leaves.
pub(super) struct CommandSession<'a> {
    pub(super) shared: &'a Shared,
    pub(super) outbox: &'a SyncSender<Frame>,
}

impl CommandSession<'_> {
    pub(super) fn run(&self, reader: &mut MessageReader) -> Result<(), ReadError> {
        while !self.shared.stop.load(Ordering::SeqCst) {
            let reply = match reader.poll::<ClientMessage>()? {
                None => continue,
                Some(ClientMessage::Command { id, command }) => ServerMessage::Ack {
                    id,
                    outcome: self.handle(command),
                },
                Some(ClientMessage::Ping) => ServerMessage::Pong,
                Some(ClientMessage::Hello { .. }) => return Ok(()),
            };
            self.outbox
                .try_send(Arc::new(encode_message(&reply)?))
                .map_err(|_| ReadError::Closed)?;
        }
        Ok(())
    }

    /// A panic in the handler rejects that command; the connection stays.
    fn handle(&self, command: protocol::Command) -> Outcome {
        catch_unwind(AssertUnwindSafe(|| self.shared.handler.handle(command))).unwrap_or_else(
            |_| Outcome::Rejected {
                reason: "handler failed".into(),
            },
        )
    }
}
