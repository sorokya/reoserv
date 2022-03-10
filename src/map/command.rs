use eo::{
    data::{EOByte, EOInt, EOShort},
    net::{packets::server::map_info, NearbyInfo},
    world::{Coords, Direction, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{player::PlayerHandle, PacketBuf};

#[derive(Debug)]
pub enum Command {
    DropPlayer(EOShort, Coords),
    Enter(EOShort, PlayerHandle),
    Face(EOShort, Direction),
    GetCharacterMapInfo {
        player_id: EOShort,
        respond_to:
            oneshot::Sender<Result<map_info::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetHashAndSize {
        respond_to: oneshot::Sender<([EOByte; 4], EOInt)>,
    },
    GetNearbyInfo {
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    Leave {
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    },
    Serialize {
        respond_to: oneshot::Sender<PacketBuf>,
    },
}
