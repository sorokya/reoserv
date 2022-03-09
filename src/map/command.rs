use eo::{
    data::{EOByte, EOInt, EOShort},
    net::NearbyInfo,
};
use tokio::sync::oneshot;

use crate::{player::PlayerHandle, PacketBuf};

#[derive(Debug)]
pub enum Command {
    Enter(EOShort, PlayerHandle),
    GetHashAndSize {
        respond_to: oneshot::Sender<([EOByte; 4], EOInt)>,
    },
    GetNearbyInfo {
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    Serialize {
        respond_to: oneshot::Sender<PacketBuf>,
    },
}
