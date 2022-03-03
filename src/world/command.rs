use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};

use super::WorldHandle;

#[derive(Debug)]
pub enum Command {
    StartListener {
        respond_to: oneshot::Sender<bool>,
    },
    AcceptConnection {
        world_handle: Arc<Mutex<WorldHandle>>,
        respond_to: oneshot::Sender<bool>,
    },
    LoadPubFiles {
        respond_to: oneshot::Sender<bool>,
    },
    LoadMapFiles {
        respond_to: oneshot::Sender<bool>,
    },
}
