use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, CharacterMapInfo, Family, Item, Spell, Weight},
    world::Coords,
};
use tokio::sync::oneshot;

use crate::{character::Character, PacketBuf};

use super::{InvalidStateError, State};

#[derive(Debug)]
pub enum Command {
    Send(Action, Family, PacketBuf),
    PongNewSequence {
        respond_to: oneshot::Sender<()>,
    },
    GenSequence {
        respond_to: oneshot::Sender<EOInt>,
    },
    Close(String),
    EnsureValidSequenceForAccountCreation {
        respond_to: oneshot::Sender<()>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetEncodingMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    GetIpAddr {
        respond_to: oneshot::Sender<String>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetCharacterId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetMapId {
        respond_to: oneshot::Sender<Result<EOShort, InvalidStateError>>,
    },
    GetCoords {
        respond_to: oneshot::Sender<Result<Coords, InvalidStateError>>,
    },
    GetWeight {
        respond_to: oneshot::Sender<Result<Weight, InvalidStateError>>,
    },
    GetItems {
        respond_to: oneshot::Sender<Result<Vec<Item>, InvalidStateError>>,
    },
    GetSpells {
        respond_to: oneshot::Sender<Result<Vec<Spell>, InvalidStateError>>,
    },
    GetCharacterMapInfo {
        respond_to: oneshot::Sender<Result<CharacterMapInfo, InvalidStateError>>,
    },
    IsInRange {
        coords: Coords,
        respond_to: oneshot::Sender<bool>,
    },
    SetAccountId(EOInt),
    SetCharacter(Character),
    SetState(State),
    SetBusy(bool),
    CalculateStats {
        respond_to: oneshot::Sender<Result<(), InvalidStateError>>,
    },
    Ping,
    Pong,
}
