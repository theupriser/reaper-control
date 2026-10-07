use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

pub(in crate::server) type Frame = Arc<Vec<u8>>;

pub(in crate::server) struct ConnectedClient {
    pub(super) id: u64,
    pub(super) outbox: SyncSender<Frame>,
    pub(super) stream: TcpStream,
}
