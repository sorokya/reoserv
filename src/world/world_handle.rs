use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{
        client,
        server::{account, character, init, login, welcome},
        FileType, OnlinePlayers,
    },
};
use mysql_async::Pool;
use tokio::sync::{mpsc, oneshot};

use crate::{character::Character, map::MapHandle, player::PlayerHandle};

use super::{world::World, Command};

#[derive(Debug, Clone)]
pub struct WorldHandle {
    tx: mpsc::UnboundedSender<Command>,
    pub is_alive: bool,
}

impl WorldHandle {
    pub fn new(pool: Pool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let world = World::new(rx, pool);
        let _ = tokio::task::Builder::new()
            .name("World")
            .spawn(run_world(world));

        Self { tx, is_alive: true }
    }

    pub async fn add_player(
        &mut self,
        player_id: EOShort,
        player: PlayerHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddPlayer {
            player_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap();
        Ok(())
    }

    pub fn broadcast_admin_message(&self, name: String, message: String) {
        let _ = self
            .tx
            .send(Command::BroadcastAdminMessage { name, message });
    }

    pub fn broadcast_announcement(&self, name: String, message: String) {
        let _ = self
            .tx
            .send(Command::BroadcastAnnouncement { name, message });
    }

    pub fn broadcast_global_message(
        &self,
        target_player_id: EOShort,
        name: String,
        message: String,
    ) {
        let _ = self.tx.send(Command::BroadcastGlobalMessage {
            target_player_id,
            name,
            message,
        });
    }

    pub fn _broadcast_server_message(&self, message: String) {
        let _ = self.tx.send(Command::_BroadcastServerMessage { message });
    }

    pub async fn create_account(
        &self,
        player: PlayerHandle,
        details: client::account::Create,
    ) -> Result<account::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CreateAccount {
            player,
            details,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn create_character(
        &self,
        details: client::character::Create,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CreateCharacter {
            details,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn delete_character(
        &self,
        session_id: EOShort,
        character_id: EOInt,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::DeleteCharacter {
            session_id,
            character_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn drop_player(
        &self,
        player_id: EOShort,
        account_id: EOInt,
        character_name: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::DropPlayer {
            respond_to: tx,
            player_id,
            account_id,
            character_name,
        });
        rx.await.unwrap();
        Ok(())
    }

    pub async fn enter_game(
        &self,
        session_id: EOShort,
        player: PlayerHandle,
    ) -> Result<welcome::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::EnterGame {
            session_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_character_by_name(
        &self,
        name: String,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacterByName {
            name,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_file(
        &self,
        file_type: FileType,
        session_id: EOShort,
        file_id: Option<EOChar>,
        player: PlayerHandle,
    ) -> Result<init::Init, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetFile {
            file_type,
            session_id,
            file_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_map(
        &self,
        map_id: EOShort,
    ) -> Result<MapHandle, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMap {
            map_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_next_player_id(
        &self,
    ) -> Result<EOShort, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNextPlayerId { respond_to: tx });
        Ok(rx.await.unwrap())
    }

    pub async fn get_online_list(&self) -> Vec<OnlinePlayers> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetOnlineList { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_player_count(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerCount { respond_to: tx });
        Ok(rx.await.unwrap())
    }

    pub async fn load_maps(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::LoadMapFiles { respond_to: tx });
        rx.await.unwrap();
    }

    pub async fn login(
        &self,
        player: PlayerHandle,
        name: String,
        password: String,
    ) -> Result<login::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Login {
            player,
            name,
            password,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn ping_players(&self) {
        let _ = self.tx.send(Command::PingPlayers);
    }

    pub fn recover_npcs(&self) {
        let _ = self.tx.send(Command::RecoverNpcs);
    }

    pub fn recover_players(&self) {
        let _ = self.tx.send(Command::RecoverPlayers);
    }

    pub fn report_player(&self, player_id: EOShort, reportee_name: String, message: String) {
        let _ = self.tx.send(Command::ReportPlayer {
            player_id,
            reportee_name,
            message,
        });
    }

    pub async fn request_account_creation(
        &self,
        name: String,
        player: PlayerHandle,
    ) -> Result<account::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestAccountCreation {
            name,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn request_character_creation(
        &self,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestCharacterCreation {
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn request_character_deletion(
        &self,
        character_id: EOInt,
        player: PlayerHandle,
    ) -> Result<character::Player, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestCharacterDeletion {
            character_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn select_character(
        &self,
        character_id: EOInt,
        player: PlayerHandle,
    ) -> Result<welcome::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::SelectCharacter {
            character_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn send_admin_message(&self, player_id: EOShort, message: String) {
        let _ = self
            .tx
            .send(Command::SendAdminMessage { player_id, message });
    }

    pub fn send_private_message(&self, from: PlayerHandle, to: String, message: String) {
        let _ = self
            .tx
            .send(Command::SendPrivateMessage { from, to, message });
    }

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Shutdown { respond_to: tx });
        rx.await.unwrap();
    }

    pub fn spawn_items(&self) {
        let _ = self.tx.send(Command::SpawnItems);
    }

    pub fn spawn_npcs(&self) {
        let _ = self.tx.send(Command::SpawnNpcs);
    }

    pub fn act_npcs(&self) {
        let _ = self.tx.send(Command::ActNpcs);
    }
}

async fn run_world(mut world: World) {
    loop {
        if let Some(command) = world.rx.recv().await {
            world.handle_command(command).await;
        }
    }
}
