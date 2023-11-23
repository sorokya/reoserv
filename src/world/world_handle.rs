use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{client, FileType, OnlinePlayers},
};
use mysql_async::Pool;
use tokio::sync::{mpsc, oneshot};

use crate::{character::Character, map::MapHandle, player::PlayerHandle};

use super::{world::World, Command, Party};

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
            .spawn(run_world(world, WorldHandle::for_tx(tx.clone())));

        Self { tx, is_alive: true }
    }

    fn for_tx(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx, is_alive: true }
    }

    pub fn accept_party_request(
        &self,
        player_id: EOShort,
        target_player_id: EOShort,
        request_type: EOChar,
    ) {
        let _ = self.tx.send(Command::AcceptPartyRequest {
            player_id,
            target_player_id,
            request_type,
        });
    }

    pub fn add_logged_in_account(&self, account_id: EOInt) {
        let _ = self.tx.send(Command::AddLoggedInAccount { account_id });
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

    pub fn broadcast_party_message(&self, player_id: EOShort, message: String) {
        let _ = self
            .tx
            .send(Command::BroadcastPartyMessage { player_id, message });
    }

    pub fn _broadcast_server_message(&self, message: String) {
        let _ = self.tx.send(Command::_BroadcastServerMessage { message });
    }

    pub fn change_password(
        &self,
        player_id: EOShort,
        username: String,
        current_password: String,
        new_password: String,
    ) {
        let _ = self.tx.send(Command::ChangePassword {
            player_id,
            username,
            current_password,
            new_password,
        });
    }

    pub fn create_account(&self, player_id: EOShort, details: client::account::Create) {
        let _ = self.tx.send(Command::CreateAccount { player_id, details });
    }

    pub fn create_character(&self, player_id: EOShort, details: client::character::Create) {
        let _ = self
            .tx
            .send(Command::CreateCharacter { player_id, details });
    }

    pub fn delete_character(&self, player_id: EOShort, session_id: EOShort, character_id: EOInt) {
        let _ = self.tx.send(Command::DeleteCharacter {
            player_id,
            session_id,
            character_id,
        });
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

    pub fn enter_game(&self, player_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::EnterGame {
            player_id,
            session_id,
        });
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

    pub fn get_file(
        &self,
        player_id: EOShort,
        file_type: FileType,
        session_id: EOShort,
        file_id: Option<EOChar>,
        warp: bool,
    ) {
        let _ = self.tx.send(Command::GetFile {
            player_id,
            file_type,
            session_id,
            file_id,
            warp,
        });
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

    pub async fn get_player_party(&self, player_id: EOShort) -> Option<Party> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerParty {
            player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn is_logged_in(&self, account_id: EOInt) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsLoggedIn {
            account_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn load_maps(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::LoadMapFiles {
            world: self.clone(),
            respond_to: tx,
        });
        rx.await.unwrap();
    }

    pub fn login(&self, player_id: EOShort, name: String, password: String) {
        let _ = self.tx.send(Command::Login {
            player_id,
            name,
            password,
        });
    }

    pub fn ping_players(&self) {
        let _ = self.tx.send(Command::PingPlayers);
    }

    pub fn report_player(&self, player_id: EOShort, reportee_name: String, message: String) {
        let _ = self.tx.send(Command::ReportPlayer {
            player_id,
            reportee_name,
            message,
        });
    }

    pub fn request_account_creation(&self, player_id: EOShort, name: String) {
        let _ = self
            .tx
            .send(Command::RequestAccountCreation { player_id, name });
    }

    pub fn request_character_creation(&self, player_id: EOShort) {
        let _ = self
            .tx
            .send(Command::RequestCharacterCreation { player_id });
    }

    pub fn request_character_deletion(&self, player_id: EOShort, character_id: EOInt) {
        let _ = self.tx.send(Command::RequestCharacterDeletion {
            player_id,
            character_id,
        });
    }

    pub fn request_party_list(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::RequestPartyList { player_id });
    }

    pub fn remove_party_member(&self, player_id: EOShort, target_player_id: EOShort) {
        let _ = self.tx.send(Command::RemovePartyMember {
            player_id,
            target_player_id,
        });
    }

    pub fn save(&self) {
        let _ = self.tx.send(Command::Save);
    }

    pub fn select_character(&self, player_id: EOShort, character_id: EOInt) {
        let _ = self.tx.send(Command::SelectCharacter {
            player_id,
            character_id,
        });
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

    pub fn tick(&self) {
        let _ = self.tx.send(Command::Tick);
    }

    pub fn update_party_hp(&self, player_id: EOShort, hp_percentage: EOChar) {
        let _ = self.tx.send(Command::UpdatePartyHP {
            player_id,
            hp_percentage,
        });
    }
}

async fn run_world(mut world: World, world_handle: WorldHandle) {
    loop {
        if let Some(command) = world.rx.recv().await {
            world.handle_command(command, world_handle.clone()).await;
        }
    }
}
