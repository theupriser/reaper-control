use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::SyncSender;

use protocol::message::{ClientMessage, Outcome, ServerMessage, encode_message};

use super::recent_answers::RecentAnswers;

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
        let mut answers = RecentAnswers::default();
        while !self.shared.stop.load(Ordering::SeqCst) {
            let Some(value) = reader.poll::<serde_json::Value>()? else {
                continue;
            };
            let reply = match serde_json::from_value::<ClientMessage>(value.clone()) {
                Err(error) => match unknown_command_id(&value) {
                    Some(id) => ServerMessage::Ack {
                        id,
                        outcome: Outcome::Rejected {
                            reason: format!("unknown command: {error}"),
                        },
                    },
                    None => return Err(protocol::message::CodecError::from(error).into()),
                },
                Ok(ClientMessage::Command { id, command }) => {
                    let outcome = match answers.find(id) {
                        Some(known) => known.clone(),
                        None => {
                            let outcome = self.handle(command);
                            answers.remember(id, outcome.clone());
                            outcome
                        }
                    };
                    ServerMessage::Ack { id, outcome }
                }
                Ok(ClientMessage::Ping) => ServerMessage::Pong,
                Ok(ClientMessage::GetCatalog) => {
                    if let Some(frame) = self.shared.hub().catalog_frame() {
                        self.outbox.try_send(frame).map_err(|_| ReadError::Closed)?;
                    }
                    continue;
                }
                Ok(ClientMessage::Hello { .. }) => return Ok(()),
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

/// The id of a well-formed Command message whose command this version does not know.
fn unknown_command_id(value: &serde_json::Value) -> Option<u64> {
    if value.get("type")?.as_str()? != "Command" {
        return None;
    }
    value.get("id")?.as_u64()
}
