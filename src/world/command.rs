use eolib::protocol::net::{
    client::{AccountCreateClientPacket, CharacterCreateClientPacket, FileType},
    server::{OnlinePlayer, PartyExpShare},
    PartyRequestType,
};
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
    ChangePassword {
        player_id: i32,
        username: String,
        current_password: String,
        new_password: String,
    },
    CreateAccount {
        player_id: i32,
        details: AccountCreateClientPacket,
    },
    CreateCharacter {
        player_id: i32,
        details: CharacterCreateClientPacket,
    },
    DeleteCharacter {
        player_id: i32,
        session_id: i32,
        character_id: i32,
    },
    DropPlayer {
        player_id: i32,
        account_id: i32,
        character_name: String,
        respond_to: oneshot::Sender<()>,
    },
    EnterGame {
        player_id: i32,
        session_id: i32,
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
    GetFile {
        player_id: i32,
        file_type: FileType,
        session_id: i32,
        file_id: Option<i32>,
        warp: bool,
    },
    GetMap {
        map_id: i32,
        respond_to: oneshot::Sender<Result<MapHandle, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<i32>,
    },
    GetOnlineList {
        respond_to: oneshot::Sender<Vec<OnlinePlayer>>,
    },
    GetPlayerParty {
        player_id: i32,
        respond_to: oneshot::Sender<Option<Party>>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<usize>,
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
    Login {
        player_id: i32,
        name: String,
        password: String,
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
    RequestAccountCreation {
        player_id: i32,
        name: String,
    },
    RequestCharacterCreation {
        player_id: i32,
    },
    RequestCharacterDeletion {
        player_id: i32,
        character_id: i32,
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
    SelectCharacter {
        player_id: i32,
        character_id: i32,
    },
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
