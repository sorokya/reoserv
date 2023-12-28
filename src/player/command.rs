use bytes::Bytes;
use eo::{
    data::{u8, i32, EOInt, i32},
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
        map_id: i32,
        session_id: i32,
    },
    CancelTrade,
    Close(String),
    ArenaDie {
        spawn_coords: Coords,
    },
    Die,
    GenerateSessionId {
        respond_to: oneshot::Sender<i32>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetBanDuration {
        respond_to: oneshot::Sender<Option<EOInt>>,
    },
    GetBoardId {
        respond_to: oneshot::Sender<Option<i32>>,
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
        respond_to: oneshot::Sender<Result<i32, InvalidStateError>>,
    },
    GetPlayerId {
        respond_to: oneshot::Sender<i32>,
    },
    GetPartyRequest {
        respond_to: oneshot::Sender<PartyRequest>,
    },
    GetSessionId {
        respond_to: oneshot::Sender<Result<i32, MissingSessionIdError>>,
    },
    GetInteractNpcIndex {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetInteractPlayerId {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(i32, i32)>,
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
        map_id: i32,
        coords: Coords,
        animation: Option<WarpAnimation>,
    },
    Send(PacketAction, PacketFamily, Bytes),
    SetAccountId(EOInt),
    SetBoardId(i32),
    SetBusy(bool),
    SetCharacter(Box<Character>),
    SetInteractNpcIndex(i32),
    SetInteractPlayerId(Option<i32>),
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
        respond_to: oneshot::Sender<Result<i32, MissingSessionIdError>>,
    },
    UpdatePartyHP {
        hp_percentage: i32,
    },
}
