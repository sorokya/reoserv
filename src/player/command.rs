use bytes::Bytes;
use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    protocol::{Coords, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
};

use super::ClientState;

#[derive(Debug)]
pub enum Command {
    AcceptWarp {
        map_id: EOShort,
        session_id: EOShort,
    },
    CancelTrade {
        player_id: EOShort,
    },
    Close(String),
    Die,
    GenerateSessionId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetBoardId {
        respond_to: oneshot::Sender<Option<EOShort>>,
    },
    GetCharacter {
        respond_to: oneshot::Sender<Result<Box<Character>, InvalidStateError>>,
    },
    GetChestIndex {
        respond_to: oneshot::Sender<Option<usize>>,
    },
    GenEncodingMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
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
    GetInteractNpcIndex {
        respond_to: oneshot::Sender<Option<EOChar>>,
    },
    GetInteractPlayerId {
        respond_to: oneshot::Sender<Option<EOShort>>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
    },
    GetState {
        respond_to: oneshot::Sender<ClientState>,
    },
    GetSleepCost {
        respond_to: oneshot::Sender<Option<EOInt>>,
    },
    IsTrading {
        respond_to: oneshot::Sender<bool>,
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
        coords: Coords,
        animation: Option<WarpAnimation>,
    },
    Send(PacketAction, PacketFamily, Bytes),
    SetAccountId(EOInt),
    SetBoardId(EOShort),
    SetBusy(bool),
    SetCharacter(Box<Character>),
    SetInteractNpcIndex(EOChar),
    SetInteractPlayerId(Option<EOShort>),
    SetTrading(bool),
    SetChestIndex(usize),
    SetMap(MapHandle),
    SetSleepCost(EOInt),
    SetState(ClientState),
    TakeCharacter {
        respond_to: oneshot::Sender<Result<Box<Character>, InvalidStateError>>,
    },
    TakeSessionId {
        respond_to: oneshot::Sender<Result<EOShort, MissingSessionIdError>>,
    },
}
