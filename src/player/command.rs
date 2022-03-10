use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, CharacterMapInfo, Family, Item, Spell, Weight},
    world::Coords,
};
use tokio::sync::oneshot;

use crate::{character::Character, PacketBuf, map::MapHandle};

use super::{InvalidStateError, State};

#[derive(Debug)]
pub enum Command {
    CalculateStats {
        respond_to: oneshot::Sender<Result<(), InvalidStateError>>,
    },
    Close(String),
    EnsureValidSequenceForAccountCreation {
        respond_to: oneshot::Sender<()>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetCharacterMapInfo {
        respond_to: oneshot::Sender<Result<CharacterMapInfo, InvalidStateError>>,
    },
    GetCoords {
        respond_to: oneshot::Sender<Result<Coords, InvalidStateError>>,
    },
    GetEncodingMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    GetIpAddr {
        respond_to: oneshot::Sender<String>,
    },
    GetItems {
        respond_to: oneshot::Sender<Result<Vec<Item>, InvalidStateError>>,
    },
    GetMap {
        respond_to: oneshot::Sender<Result<MapHandle, InvalidStateError>>,
    },
    GetMapId {
        respond_to: oneshot::Sender<Result<EOShort, InvalidStateError>>,
    },
    GetPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
    },
    GetSpells {
        respond_to: oneshot::Sender<Result<Vec<Spell>, InvalidStateError>>,
    },
    GetWeight {
        respond_to: oneshot::Sender<Result<Weight, InvalidStateError>>,
    },
    GenSequence {
        respond_to: oneshot::Sender<EOInt>,
    },
    IsInRange {
        coords: Coords,
        respond_to: oneshot::Sender<bool>,
    },
    Send(Action, Family, PacketBuf),
    SetAccountId(EOInt),
    SetBusy(bool),
    SetCharacter(Character),
    SetMap(MapHandle),
    SetState(State),
    Ping,
    Pong,
    PongNewSequence {
        respond_to: oneshot::Sender<()>,
    },
}
