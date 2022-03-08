use eo::{
    data::{EOByte, EOInt, EOShort},
    net::NearbyInfo,
};
use tokio::sync::oneshot;

use crate::{player::PlayerHandle, PacketBuf};

#[derive(Debug)]
pub enum Command {
    GetHashAndSize {
        respond_to: oneshot::Sender<([EOByte; 4], EOInt)>,
    },
    Serialize {
        respond_to: oneshot::Sender<PacketBuf>,
    },
    Enter(EOShort, PlayerHandle),
    GetNearbyInfo {
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
}
