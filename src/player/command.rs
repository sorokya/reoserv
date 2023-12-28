use bytes::Bytes;
use eo::{
    data::{u8, i32, EOInt, EOShort},
    protocol::{Coords, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
};

use super::{ClientState, PartyRequest};

#[derive(Debug)]
pub enum Command {
    AcceptWarp {
        map_id: EOShort,
        session_id: EOShort,
    },
    CancelTrade,
    Close(String),
    ArenaDie {
        spawn_coords: Coords,
    },
    Die,
    GenerateSessionId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetBanDuration {
        respond_to: oneshot::Sender<Option<EOInt>>,
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
        respond_to: oneshot::Sender<[u8; 2]>,
    },
    GetEncodingMultiples {
        respond_to: oneshot::Sender<[u8; 2]>,
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
    GetPartyRequest {
        respond_to: oneshot::Sender<PartyRequest>,
    },
    GetSessionId {
        respond_to: oneshot::Sender<Result<EOShort, MissingSessionIdError>>,
    },
    GetInteractNpcIndex {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetInteractPlayerId {
        respond_to: oneshot::Sender<Option<EOShort>>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, i32)>,
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
    IsTradeAccepted {
        respond_to: oneshot::Sender<bool>,
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
    SetInteractNpcIndex(i32),
    SetInteractPlayerId(Option<EOShort>),
    SetPartyRequest(PartyRequest),
    SetTradeAccepted(bool),
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
    UpdatePartyHP {
        hp_percentage: i32,
    },
}
