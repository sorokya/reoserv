use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, Family},
    world::TinyCoords,
};
use tokio::sync::oneshot;

use crate::{character::Character, map::MapHandle, PacketBuf};

use super::{InvalidStateError, State};

#[derive(Debug)]
pub enum Command {
    AcceptWarp {
        map_id: EOShort,
        warp_id: EOShort,
    },
    Close(String),
    EnsureValidSequenceForAccountCreation {
        respond_to: oneshot::Sender<()>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
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
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
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
    SetCharacter(Character),
    SetMap(MapHandle),
    SetState(State),
    TakeCharacter {
        respond_to: oneshot::Sender<Result<Character, InvalidStateError>>,
    },
}
