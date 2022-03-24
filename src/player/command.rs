use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, Family},
    world::TinyCoords,
};
use tokio::sync::oneshot;

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
    PacketBuf,
};

use super::State;

#[derive(Debug)]
pub enum Command {
    AcceptWarp {
        map_id: EOShort,
        session_id: EOShort,
    },
    Close(String),
    EnsureValidSequenceForAccountCreation {
        respond_to: oneshot::Sender<()>,
    },
    GenerateSessionId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetCharacter {
        respond_to: oneshot::Sender<Result<Character, InvalidStateError>>,
    },
    GetEncodingMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    GetIpAddr {
        respond_to: oneshot::Sender<String>,
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
    GetSessionId {
        respond_to: oneshot::Sender<Result<EOShort, MissingSessionIdError>>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
    },
    GetState {
        respond_to: oneshot::Sender<State>,
    },
    GenSequence {
        respond_to: oneshot::Sender<EOInt>,
    },
    Ping,
    Pong,
    PongNewSequence {
        respond_to: oneshot::Sender<()>,
    },
    RequestWarp {
        local: bool,
        map_id: EOShort,
        coords: TinyCoords,
    },
    Send(Action, Family, PacketBuf),
    SetAccountId(EOInt),
    SetBusy(bool),
    SetCharacter(Box<Character>),
    SetMap(MapHandle),
    SetState(State),
    TakeSessionId {
        respond_to: oneshot::Sender<Result<EOShort, MissingSessionIdError>>,
    },
}
