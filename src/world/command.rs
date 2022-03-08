use eo::{
    data::{
        pubs::{ClassRecord, ItemRecord},
        EOChar, EOInt, EOShort,
    },
    net::{
        packets::server::{account, character, init, login, welcome},
        FileType,
    },
};
use tokio::sync::oneshot;

use crate::player::{Player, PlayerHandle};

#[derive(Debug)]
pub enum Command {
    LoadPubFiles {
        respond_to: oneshot::Sender<()>,
    },
    LoadMapFiles {
        respond_to: oneshot::Sender<()>,
    },
    StartPingTimer {
        respond_to: oneshot::Sender<()>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<usize>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    AddPlayer {
        respond_to: oneshot::Sender<()>,
        player_id: EOShort,
        player: PlayerHandle,
    },
    DropPlayer {
        player_id: EOShort,
        account_id: EOInt,
        character_id: EOInt,
        respond_to: oneshot::Sender<()>,
    },
    RequestAccountCreation {
        name: String,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<account::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    CreateAccount {
        details: eo::net::packets::client::account::Create,
        register_ip: String,
        respond_to:
            oneshot::Sender<Result<account::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    Login {
        name: String,
        password: String,
        player: PlayerHandle,
        respond_to: oneshot::Sender<Result<login::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    RequestCharacterCreation {
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    CreateCharacter {
        details: eo::net::packets::client::character::Create,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    RequestCharacterDeletion {
        character_id: EOInt,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Player, Box<dyn std::error::Error + Send + Sync>>>,
    },
    DeleteCharacter {
        session_id: EOShort,
        character_id: EOInt,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    SelectCharacter {
        character_id: EOInt,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<welcome::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetFile {
        file_type: FileType,
        player: PlayerHandle,
        respond_to: oneshot::Sender<Result<init::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    EnterGame {
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<welcome::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetClass {
        class_id: EOChar,
        respond_to: oneshot::Sender<Result<ClassRecord, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetItem {
        item_id: EOShort,
        respond_to: oneshot::Sender<Result<ItemRecord, Box<dyn std::error::Error + Send + Sync>>>,
    },
}
