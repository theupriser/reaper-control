use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

pub(super) type Frame = Arc<Vec<u8>>;

pub(super) struct Client {
    pub(super) id: u64,
    pub(super) outbox: SyncSender<Frame>,
    pub(super) stream: TcpStream,
}
