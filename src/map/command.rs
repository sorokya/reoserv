use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{server::range, Coords, Direction, Emote, NearbyInfo, ShortItem, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::character::Character;

#[derive(Debug)]
pub enum Command {
    Attack {
        target_player_id: EOShort,
        direction: Direction,
    },
    DropItem {
        target_player_id: EOShort,
        item: ShortItem,
        coords: Coords,
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
    Equip {
        player_id: EOShort,
        item_id: EOShort,
        sub_loc: EOChar,
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
    GiveItem {
        target_player_id: EOShort,
        item_id: EOShort,
        amount: EOInt,
    },
    JunkItem {
        target_player_id: EOShort,
        item_id: EOShort,
        amount: EOInt,
    },
    Leave {
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
    },
    OpenChest {
        player_id: EOShort,
        coords: Coords,
    },
    OpenDoor {
        target_player_id: EOShort, // TODO: rename to player_id
        door_coords: Coords,       // TODO: rename to coords
    },
    RecoverNpcs,
    RecoverPlayers,
    RequestPaperdoll {
        player_id: EOShort,
        target_player_id: EOShort,
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
    Sit {
        player_id: EOShort,
    },
    Stand {
        player_id: EOShort,
    },
    TakeChestItem {
        player_id: EOShort,
        coords: Coords,
        item_id: EOShort,
    },
    Unequip {
        player_id: EOShort,
        item_id: EOShort,
        sub_loc: EOChar,
    },
    UseItem {
        player_id: EOShort,
        item_id: EOShort,
    },
    Walk {
        target_player_id: EOShort,
        direction: Direction,
    },
    SpawnItems,
    SpawnNpcs,
    ActNpcs,
}
