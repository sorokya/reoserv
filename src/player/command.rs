use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, Family},
};
use tokio::sync::oneshot;

use crate::{character::Character, map::MapHandle, PacketBuf};

use super::{InvalidStateError, State};

#[derive(Debug)]
pub enum Command {
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
