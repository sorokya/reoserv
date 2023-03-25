use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{server::range, Coords, Direction, Emote, NearbyInfo, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{character::Character};

#[derive(Debug)]
pub enum Command {
    Attack {
        target_player_id: EOShort,
        direction: Direction,
    },
    Emote {
        target_player_id: EOShort,
        emote: Emote,
    },
    Enter {
        character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    },
    Face {
        target_player_id: EOShort,
        direction: Direction,
    },
    GetCharacter {
        player_id: EOShort,
        respond_to: oneshot::Sender<Option<Box<Character>>>,
    },
    GetDimensions {
        respond_to: oneshot::Sender<(EOChar, EOChar)>,
    },
    GetItem {
        target_player_id: EOShort,
        item_index: EOShort,
    },
    GetMapInfo {
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
        respond_to: oneshot::Sender<range::Reply>,
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
        door_coords: Coords,
    },
    Save {
        respond_to: oneshot::Sender<()>,
    },
    SendChatMessage {
        target_player_id: EOShort,
        message: String,
    },
    Serialize {
        respond_to: oneshot::Sender<Bytes>,
    },
    Walk {
        target_player_id: EOShort,
        direction: Direction,
    },
    SpawnNpcs,
    ActNpcs,
}
