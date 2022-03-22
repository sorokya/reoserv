use eo::{
    character::Emote,
    data::{EOChar, EOInt, EOShort, EOThree},
    net::{packets::server::map_info, NearbyInfo},
    world::{Direction, TinyCoords, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{character::Character, PacketBuf};

#[derive(Debug)]
pub enum Command {
    Emote {
        target_player_id: EOShort,
        emote: Emote,
    },
    Enter {
        character: Box<Character>,
        respond_to: oneshot::Sender<()>,
    },
    Face {
        target_player_id: EOShort,
        direction: Direction,
    },
    GetMapInfo {
        player_ids: Option<Vec<EOShort>>,
        npc_indexes: Option<Vec<EOChar>>,
        respond_to: oneshot::Sender<map_info::Reply>,
    },
    GetNearbyInfo {
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    GetRidAndSize {
        respond_to: oneshot::Sender<([EOShort; 2], EOInt)>,
    },
    Leave {
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
    },
    OpenDoor {
        target_player_id: EOShort,
        door_coords: TinyCoords,
    },
    Serialize {
        respond_to: oneshot::Sender<PacketBuf>,
    },
    Walk {
        target_player_id: EOShort,
        timestamp: EOThree,
        coords: TinyCoords,
        direction: Direction,
    },
}
