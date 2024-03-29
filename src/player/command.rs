use bytes::Bytes;
use eolib::protocol::{
    net::{
        client::{
            AccountCreateClientPacket, CharacterCreateClientPacket, FileType,
            GuildAgreeClientPacketInfoTypeData, GuildInfoType,
        },
        server::WarpEffect,
        PacketAction, PacketFamily, Version,
    },
    Coords,
};
use eoplus::Arg;
use tokio::sync::oneshot;

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
};

use super::{ClientState, PartyRequest, PlayerHandle};

#[derive(Debug)]
pub enum Command {
    AcceptGuildJoinRequest {
        player_id: i32,
    },
    AcceptWarp {
        map_id: i32,
        session_id: i32,
    },
    AddGuildCreationPlayer {
        player_id: i32,
        name: String,
    },
    BeginHandshake {
        challenge: i32,
        hdid: String,
        version: Version,
    },
    CancelTrade,
    ChangePassword {
        username: String,
        old_password: String,
        new_password: String,
    },
    Close(String),
    CreateAccount(AccountCreateClientPacket),
    CreateCharacter(CharacterCreateClientPacket),
    CreateGuild {
        session_id: i32,
        guild_name: String,
        guild_tag: String,
        guild_description: String,
    },
    ArenaDie {
        spawn_coords: Coords,
    },
    CompleteHandshake {
        player_id: i32,
        client_encryption_multiple: i32,
        server_encryption_multiple: i32,
    },
    DeleteCharacter {
        session_id: i32,
        character_id: i32,
    },
    Die,
    DisbandGuild {
        session_id: i32,
    },
    EnterGame {
        session_id: i32,
    },
    GenerateSessionId {
        respond_to: oneshot::Sender<i32>,
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
    GetFile {
        file_type: FileType,
        session_id: i32,
        file_id: Option<i32>,
        warp: bool,
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
    GetSessionId {
        respond_to: oneshot::Sender<Result<i32, MissingSessionIdError>>,
    },
    GetInteractNpcIndex {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetInteractPlayerId {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    GetState {
        respond_to: oneshot::Sender<ClientState>,
    },
    GetSleepCost {
        respond_to: oneshot::Sender<Option<i32>>,
    },
    IsTradeAccepted {
        respond_to: oneshot::Sender<bool>,
    },
    IsTrading {
        respond_to: oneshot::Sender<bool>,
    },
    GenSequence {
        respond_to: oneshot::Sender<i32>,
    },
    KickGuildMember {
        session_id: i32,
        member_name: String,
    },
    LeaveGuild {
        session_id: i32,
    },
    Login {
        username: String,
        password: String,
    },
    Pong,
    PongNewSequence {
        respond_to: oneshot::Sender<()>,
    },
    QuestAction {
        action: String,
        args: Vec<Arg>,
    },
    RequestAccountCreation {
        username: String,
    },
    RequestCharacterCreation,
    RequestCharacterDeletion {
        character_id: i32,
    },
    RequestGuildCreation {
        session_id: i32,
        guild_name: String,
        guild_tag: String,
    },
    RequestGuildDetails {
        session_id: i32,
        guild_identity: String,
    },
    RequestGuildMemberlist {
        session_id: i32,
        guild_identity: String,
    },
    RequestGuildInfo {
        session_id: i32,
        info_type: GuildInfoType,
    },
    RequestWarp {
        local: bool,
        map_id: i32,
        coords: Coords,
        animation: Option<WarpEffect>,
    },
    SelectCharacter {
        player_handle: PlayerHandle,
        character_id: i32,
    },
    Send(PacketAction, PacketFamily, Bytes),
    SetBoardId(i32),
    SetBusy(bool),
    SetInteractNpcIndex(i32),
    SetInteractPlayerId(Option<i32>),
    SetPartyRequest(PartyRequest),
    SetTradeAccepted(bool),
    SetTrading(bool),
    SetChestIndex(usize),
    SetSleepCost(i32),
    Tick,
    UpdateGuild {
        session_id: i32,
        info_type_data: GuildAgreeClientPacketInfoTypeData,
    },
    AssignGuildRank {
        session_id: i32,
        member_name: String,
        rank: i32,
    },
    UpdatePartyHP {
        hp_percentage: i32,
    },
}
