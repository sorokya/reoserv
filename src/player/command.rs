use bytes::Bytes;
use eolib::protocol::{
    net::{
        server::{GuildReply, WarpEffect},
        PacketAction, PacketFamily,
    },
    Coords,
};
use eoplus::Arg;
use tokio::sync::oneshot;

use crate::{character::Character, errors::InvalidStateError, map::MapHandle};

use super::{ClientState, PartyRequest};

#[derive(Debug)]
pub enum Command {
    AddGuildCreationPlayer {
        player_id: i32,
        name: String,
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
    GetCharacter {
        respond_to: oneshot::Sender<Result<Box<Character>, InvalidStateError>>,
    },
    GetChestIndex {
        respond_to: oneshot::Sender<Option<usize>>,
    },
    GetMap {
        respond_to: oneshot::Sender<Result<MapHandle, InvalidStateError>>,
    },
    GetPlayerId {
        respond_to: oneshot::Sender<i32>,
    },
    GetPartyRequest {
        respond_to: oneshot::Sender<PartyRequest>,
    },
    GetInteractPlayerId {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetState {
        respond_to: oneshot::Sender<ClientState>,
    },
    IsTradeAccepted {
        respond_to: oneshot::Sender<bool>,
    },
    IsTrading {
        respond_to: oneshot::Sender<bool>,
    },
    QuestAction {
        action: String,
        args: Vec<Arg>,
    },
    RequestWarp {
        local: bool,
        map_id: i32,
        coords: Coords,
        animation: Option<WarpEffect>,
    },
    SendGuildReply(GuildReply),
    SendServerMessage(String),
    Send(PacketAction, PacketFamily, Bytes),
    SetBoardId(i32),
    SetInteractNpcIndex(i32),
    SetInteractPlayerId(Option<i32>),
    SetPartyRequest(PartyRequest),
    SetTradeAccepted(bool),
    SetTrading(bool),
    SetChestIndex(usize),
    SetSleepCost(i32),
    Tick,
    UpdatePartyHP {
        hp_percentage: i32,
    },
}
