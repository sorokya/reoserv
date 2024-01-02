use chrono::{DateTime, Utc};
use eolib::protocol::net::{server::PartyExpShare, PartyRequestType};
use tokio::sync::oneshot;

use crate::{character::Character, map::MapHandle, player::PlayerHandle};

use super::{Party, WorldHandle};

#[derive(Debug)]
pub enum Command {
    AcceptPartyRequest {
        player_id: i32,
        target_player_id: i32,
        request_type: PartyRequestType,
    },
    AddCharacter {
        player_id: i32,
        name: String,
    },
    AddConnection {
        ip: String,
        respond_to: oneshot::Sender<()>,
    },
    AddLoggedInAccount {
        account_id: i32,
    },
    AddPlayer {
        respond_to: oneshot::Sender<()>,
        player_id: i32,
        player: PlayerHandle,
    },
    BanPlayer {
        victim_name: String,
        admin_name: String,
        duration: String,
        silent: bool,
    },
    BroadcastAdminMessage {
        name: String,
        message: String,
    },
    BroadcastAnnouncement {
        name: String,
        message: String,
    },
    BroadcastGlobalMessage {
        target_player_id: i32,
        name: String,
        message: String,
    },
    _BroadcastServerMessage {
        message: String,
    },
    BroadcastPartyMessage {
        player_id: i32,
        message: String,
    },
    DropPlayer {
        player_id: i32,
        ip: String,
        account_id: i32,
        character_name: String,
        respond_to: oneshot::Sender<()>,
    },
    FindPlayer {
        player_id: i32,
        name: String,
    },
    FreePlayer {
        victim_name: String,
    },
    FreezePlayer {
        victim_name: String,
        admin_name: String,
    },
    GetCharacterByName {
        name: String,
        respond_to:
            oneshot::Sender<Result<Box<Character>, Box<dyn std::error::Error + Sync + Send>>>,
    },
    GetConnectionCount {
        respond_to: oneshot::Sender<i32>,
    },
    GetIpConnectionCount {
        ip: String,
        respond_to: oneshot::Sender<i32>,
    },
    GetIpLastConnect {
        ip: String,
        respond_to: oneshot::Sender<Option<DateTime<Utc>>>,
    },
    GetMap {
        map_id: i32,
        respond_to: oneshot::Sender<Result<MapHandle, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<i32>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<i32>,
    },
    GetPlayerParty {
        player_id: i32,
        respond_to: oneshot::Sender<Option<Party>>,
    },
    IsLoggedIn {
        account_id: i32,
        respond_to: oneshot::Sender<bool>,
    },
    JailPlayer {
        victim_name: String,
        admin_name: String,
    },
    KickPlayer {
        victim_name: String,
        admin_name: String,
        silent: bool,
    },
    LoadMapFiles {
        world: WorldHandle,
        respond_to: oneshot::Sender<()>,
    },
    MutePlayer {
        victim_name: String,
        admin_name: String,
    },
    PingPlayers,
    Quake {
        magnitude: i32,
    },
    ReportPlayer {
        player_id: i32,
        reportee_name: String,
        message: String,
    },
    RequestPartyList {
        player_id: i32,
    },
    RequestPlayerInfo {
        player_id: i32,
        victim_name: String,
    },
    RequestPlayerList {
        player_id: i32,
    },
    RequestPlayerNameList {
        player_id: i32,
    },
    RequestPlayerInventory {
        player_id: i32,
        victim_name: String,
    },
    RemovePartyMember {
        player_id: i32,
        target_player_id: i32,
    },
    Save,
    SendAdminMessage {
        player_id: i32,
        message: String,
    },
    SendPrivateMessage {
        from: PlayerHandle,
        to: String,
        message: String,
    },
    Shutdown {
        respond_to: oneshot::Sender<()>,
    },
    Tick,
    ToggleGlobal {
        admin_name: String,
    },
    UnfreezePlayer {
        victim_name: String,
        admin_name: String,
    },
    UpdatePartyHP {
        player_id: i32,
        hp_percentage: i32,
    },
    UpdatePartyExp {
        player_id: i32,
        exp_gains: Vec<PartyExpShare>,
    },
}
