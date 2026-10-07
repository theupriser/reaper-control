use protocol::AppState;

use super::connected_client::Client;

#[derive(Default)]
pub(super) struct Hub {
    pub(super) state: AppState,
    pub(super) clients: Vec<Client>,
    pub(super) next_id: u64,
}
