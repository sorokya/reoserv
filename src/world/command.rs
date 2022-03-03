use eo::data::EOShort;
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

#[derive(Debug)]
pub enum Command {
    LoadPubFiles {
        respond_to: oneshot::Sender<()>,
    },
    LoadMapFiles {
        respond_to: oneshot::Sender<()>,
    },
    StartPingTimer {
        respond_to: oneshot::Sender<()>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<usize>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    AddPlayer {
        respond_to: oneshot::Sender<()>,
        player_id: EOShort,
        player: PlayerHandle,
    },
    DropPlayer {
        player_id: EOShort,
        respond_to: oneshot::Sender<()>,
    },
}
