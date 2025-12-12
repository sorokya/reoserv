use chrono::{DateTime, Utc};
use eolib::protocol::net::{server::PartyExpShare, PartyRequestType};
use mysql_async::Pool;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

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
        tokio::spawn(run_world(world));

        Self { tx, is_alive: true }
    }

    pub fn accept_party_request(
        &self,
        player_id: i32,
        target_player_id: i32,
        request_type: PartyRequestType,
    ) {
        let _ = self.tx.send(Command::AcceptPartyRequest {
            player_id,
            target_player_id,
            request_type,
        });
    }

    pub fn add_logged_in_account(&self, account_id: i32) {
        let _ = self.tx.send(Command::AddLoggedInAccount { account_id });
    }

    pub fn add_pending_login(&self, account_id: i32) {
        let _ = self.tx.send(Command::AddPendingLogin { account_id });
    }

    pub fn remove_pending_login(&self, account_id: i32) {
        let _ = self.tx.send(Command::RemovePendingLogin { account_id });
    }

    pub fn add_character(&self, player_id: i32, name: String, guild_tag: Option<String>) {
        let _ = self.tx.send(Command::AddCharacter {
            player_id,
            name,
            guild_tag,
        });
    }

    pub fn add_guild_member(&self, player_id: i32, guild_tag: String) {
        let _ = self.tx.send(Command::AddGuildMember {
            player_id,
            guild_tag,
        });
    }

    pub async fn add_connection(&self, ip: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddConnection {
            ip: ip.to_string(),
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to add connection. Timeout".to_string())?
            .map_err(|_| "Failed to add connection. Channel closed".to_string())
    }

    pub async fn add_player(
        &mut self,
        player_id: i32,
        player: PlayerHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddPlayer {
            player_id,
            player,
            respond_to: tx,
        });
        timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to add player. Timeout".into() })?
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to add player. Channel closed".into() })?;
        Ok(())
    }

    pub fn ban_player(
        &self,
        victim_name: String,
        duration: String,
        admin_name: String,
        silent: bool,
    ) {
        let _ = self.tx.send(Command::BanPlayer {
            victim_name,
            duration,
            admin_name,
            silent,
        });
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

    pub fn broadcast_global_message(&self, player_id: i32, name: String, message: String) {
        let _ = self.tx.send(Command::BroadcastGlobalMessage {
            player_id,
            name,
            message,
        });
    }

    pub fn broadcast_party_message(&self, player_id: i32, message: String) {
        let _ = self
            .tx
            .send(Command::BroadcastPartyMessage { player_id, message });
    }

    pub fn broadcast_guild_message(
        &self,
        player_id: Option<i32>,
        guild_tag: String,
        name: String,
        message: String,
    ) {
        let _ = self.tx.send(Command::BroadcastGuildMessage {
            player_id,
            guild_tag,
            name,
            message,
        });
    }

    pub fn disband_guild(&self, guild_tag: String) {
        let _ = self.tx.send(Command::DisbandGuild { guild_tag });
    }

    pub async fn drop_player(
        &self,
        player_id: i32,
        ip: String,
        account_id: i32,
        character_name: String,
        guild_tag: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::DropPlayer {
            respond_to: tx,
            player_id,
            ip,
            account_id,
            character_name,
            guild_tag,
        });
        timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to drop player. Timeout".into() })?
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to drop player. Channel closed".into() })?;
        Ok(())
    }

    pub fn find_player(&self, player_id: i32, name: String) {
        let _ = self.tx.send(Command::FindPlayer { player_id, name });
    }

    pub fn free_player(&self, victim_name: String) {
        let _ = self.tx.send(Command::FreePlayer { victim_name });
    }

    pub fn freeze_player(&self, victim_name: String, admin_name: String) {
        let _ = self.tx.send(Command::FreezePlayer {
            victim_name,
            admin_name,
        });
    }

    pub async fn get_character_by_name(
        &self,
        name: &str,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacterByName {
            name: name.to_owned(),
            respond_to: tx,
        });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Failed to get character by name. Channel closed".into()),
            Err(_) => Err("Failed to get character by name. Timeout".into()),
        }
    }

    pub async fn get_map(
        &self,
        map_id: i32,
    ) -> Result<MapHandle, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMap {
            map_id,
            respond_to: tx,
        });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Failed to get map. Channel closed".into()),
            Err(_) => Err("Failed to get map. Timeout".into()),
        }
    }

    pub async fn get_next_player_id(
        &self,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNextPlayerId { respond_to: tx });
        match timeout(Duration::from_secs(1), rx).await {
            Ok(Ok(id)) => Ok(id),
            Ok(Err(_)) => Err("Failed to get next player id. Channel closed".into()),
            Err(_) => Err("Failed to get next player id. Timeout".into()),
        }
    }

    pub async fn get_connection_count(&self) -> Result<i32, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetConnectionCount { respond_to: tx });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get connection count. Timeout".to_string())?
            .map_err(|_| "Failed to get connection count. Channel closed".to_string())
    }

    pub async fn get_ip_connection_count(&self, ip: &str) -> Result<i32, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetIpConnectionCount {
            ip: ip.to_string(),
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get IP connection count. Timeout".to_string())?
            .map_err(|_| "Failed to get IP connection count. Channel closed".to_string())
    }

    pub async fn get_ip_last_connect(&self, ip: &str) -> Result<Option<DateTime<Utc>>, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetIpLastConnect {
            ip: ip.to_string(),
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get IP last connect. Timeout".to_string())?
            .map_err(|_| "Failed to get IP last connect. Channel closed".to_string())
    }

    pub async fn get_player(&self, player_id: i32) -> Result<Option<PlayerHandle>, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayer {
            player_id,
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get player. Timeout".to_string())?
            .map_err(|_| "Failed to get player. Channel closed".to_string())
    }

    pub async fn get_player_count(&self) -> Result<i32, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerCount { respond_to: tx });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get player count. Timeout".to_string())?
            .map_err(|_| "Failed to get player count. Channel closed".to_string())
    }

    pub async fn get_player_party(&self, player_id: i32) -> Result<Option<Party>, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerParty {
            player_id,
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to get player party. Timeout".to_string())?
            .map_err(|_| "Failed to get player party. Channel closed".to_string())
    }

    pub async fn is_logged_in(&self, account_id: i32) -> Result<bool, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsLoggedIn {
            account_id,
            respond_to: tx,
        });
        timeout(Duration::from_secs(1), rx)
            .await
            .map_err(|_| "Failed to check if logged in. Timeout".to_string())?
            .map_err(|_| "Failed to check if logged in. Channel closed".to_string())
    }

    pub fn jail_player(&self, victim_name: String, admin_name: String) {
        let _ = self.tx.send(Command::JailPlayer {
            victim_name,
            admin_name,
        });
    }

    pub fn kick_player(&self, victim_name: String, admin_name: String, silent: bool) {
        let _ = self.tx.send(Command::KickPlayer {
            victim_name,
            admin_name,
            silent,
        });
    }

    pub async fn load_maps(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::LoadMapFiles {
            world: self.clone(),
            respond_to: tx,
        });
        timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| "Failed to load maps. Timeout".to_string())?
            .map_err(|_| "Failed to load maps. Channel closed".to_string())
    }

    pub fn mute_player(&self, victim_name: String, admin_name: String) {
        let _ = self.tx.send(Command::MutePlayer {
            victim_name,
            admin_name,
        });
    }

    pub fn quake(&self, magnitude: i32) {
        let _ = self.tx.send(Command::Quake { magnitude });
    }

    pub fn report_player(&self, player_id: i32, reportee_name: String, message: String) {
        let _ = self.tx.send(Command::ReportPlayer {
            player_id,
            reportee_name,
            message,
        });
    }

    pub fn request_party_list(&self, player_id: i32) {
        let _ = self.tx.send(Command::RequestPartyList { player_id });
    }

    pub fn remove_guild_member(&self, player_id: i32, guild_tag: String) {
        let _ = self.tx.send(Command::RemoveGuildMember {
            player_id,
            guild_tag,
        });
    }

    pub fn remove_party_member(&self, player_id: i32, target_player_id: i32) {
        let _ = self.tx.send(Command::RemovePartyMember {
            player_id,
            target_player_id,
        });
    }

    pub fn request_player_info(&self, player_id: i32, victim_name: String) {
        let _ = self.tx.send(Command::RequestPlayerInfo {
            player_id,
            victim_name,
        });
    }

    pub fn request_player_inventory(&self, player_id: i32, victim_name: String) {
        let _ = self.tx.send(Command::RequestPlayerInventory {
            player_id,
            victim_name,
        });
    }

    pub fn request_player_name_list(&self, player_id: i32) {
        let _ = self.tx.send(Command::RequestPlayerNameList { player_id });
    }

    pub fn request_player_list(&self, player_id: i32) {
        let _ = self.tx.send(Command::RequestPlayerList { player_id });
    }

    pub fn reload_map(&self, map_id: i32) {
        let _ = self.tx.send(Command::ReloadMap { map_id });
    }

    pub fn save(&self) {
        let _ = self.tx.send(Command::Save);
    }

    pub fn send_admin_message(&self, player_id: i32, message: String) {
        let _ = self
            .tx
            .send(Command::SendAdminMessage { player_id, message });
    }

    pub fn send_private_message(&self, player_id: i32, to: String, message: String) {
        let _ = self.tx.send(Command::SendPrivateMessage {
            player_id,
            to,
            message,
        });
    }

    pub fn show_captcha(&self, victim_name: String, experience: i32) {
        let _ = self.tx.send(Command::ShowCaptcha {
            victim_name,
            experience,
        });
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Shutdown { respond_to: tx });
        timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| "Failed to shutdown. Timeout".to_string())?
            .map_err(|_| "Failed to shutdown. Channel closed".to_string())
    }

    pub fn tick(&self) {
        let _ = self.tx.send(Command::Tick);
    }

    pub fn toggle_global(&self, admin_name: String) {
        let _ = self.tx.send(Command::ToggleGlobal { admin_name });
    }

    pub fn unfreeze_player(&self, victim_name: String, admin_name: String) {
        let _ = self.tx.send(Command::UnfreezePlayer {
            victim_name,
            admin_name,
        });
    }

    pub fn update_party_hp(&self, player_id: i32, hp_percentage: i32) {
        let _ = self.tx.send(Command::UpdatePartyHP {
            player_id,
            hp_percentage,
        });
    }

    pub fn update_party_exp(&self, player_id: i32, exp_gains: Vec<PartyExpShare>) {
        let _ = self.tx.send(Command::UpdatePartyExp {
            player_id,
            exp_gains,
        });
    }
}

async fn run_world(mut world: World) {
    loop {
        if let Some(command) = world.rx.recv().await {
            world.handle_command(command).await;
        }
    }
}
