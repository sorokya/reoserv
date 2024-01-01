use crate::{errors::DataNotFoundError, map::MapHandle, player::PlayerHandle};

use super::{load_maps::load_maps, Command, Party, WorldHandle};
use mysql_async::Pool;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct World {
    pub rx: UnboundedReceiver<Command>,
    players: HashMap<i32, PlayerHandle>,
    accounts: Vec<i32>,
    characters: HashMap<String, i32>,
    pool: Pool,
    maps: Option<HashMap<i32, MapHandle>>,
    parties: Vec<Party>,
    npc_act_ticks: i32,
    npc_spawn_ticks: i32,
    item_spawn_ticks: i32,
    player_recover_ticks: i32,
    npc_recover_ticks: i32,
    quake_ticks: i32,
    spike_ticks: i32,
    drain_ticks: i32,
    warp_suck_ticks: i32,
    arena_ticks: i32,
    door_close_ticks: i32,
    global_locked: bool,
}

mod account;
mod add_player;
mod admin;
mod chat;
mod drop_player;
mod enter_game;
mod find_player;
mod get_character_by_name;
mod get_file;
mod get_next_player_id;
mod get_welcome_request_data;
mod party;
mod request_player_list;
mod request_player_name_list;
mod save;
mod shutdown;
mod tick;

impl World {
    pub fn new(rx: UnboundedReceiver<Command>, pool: Pool) -> Self {
        Self {
            rx,
            pool,
            players: HashMap::new(),
            accounts: Vec::new(),
            characters: HashMap::new(),
            maps: None,
            parties: Vec::new(),
            npc_act_ticks: 0,
            npc_spawn_ticks: 0,
            item_spawn_ticks: 0,
            player_recover_ticks: 0,
            npc_recover_ticks: 0,
            quake_ticks: 0,
            spike_ticks: 0,
            drain_ticks: 0,
            warp_suck_ticks: 0,
            arena_ticks: 0,
            door_close_ticks: 0,
            global_locked: false,
        }
    }

    pub async fn handle_command(&mut self, command: Command, world_handle: WorldHandle) {
        match command {
            Command::AcceptPartyRequest {
                player_id,
                target_player_id,
                request_type,
            } => {
                self.accept_party_request(player_id, target_player_id, request_type)
                    .await
            }

            Command::AddLoggedInAccount { account_id } => {
                self.accounts.push(account_id);
            }

            Command::AddPlayer {
                respond_to,
                player_id,
                player,
            } => self.add_player(player_id, player, respond_to),

            Command::BroadcastAdminMessage { name, message } => {
                self.broadcast_admin_message(&name, &message).await
            }

            Command::BroadcastAnnouncement { name, message } => {
                self.broadcast_announcement(&name, &message).await
            }

            Command::BroadcastGlobalMessage {
                target_player_id,
                name,
                message,
            } => {
                self.broadcast_global_message(target_player_id, &name, &message)
                    .await
            }

            Command::BroadcastPartyMessage { player_id, message } => {
                self.broadcast_party_message(player_id, message);
            }

            Command::_BroadcastServerMessage { message } => self.broadcast_server_message(&message),

            Command::ChangePassword {
                player_id,
                username,
                current_password,
                new_password,
            } => self.change_password(player_id, username, current_password, new_password),

            Command::CreateAccount { player_id, details } => {
                self.create_account(player_id, details).await
            }

            Command::CreateCharacter { player_id, details } => {
                self.create_character(player_id, details).await
            }

            Command::DeleteCharacter {
                player_id,
                session_id,
                character_id,
            } => self.delete_character(player_id, session_id, character_id),

            Command::DropPlayer {
                player_id,
                account_id,
                character_name,
                respond_to,
            } => self.drop_player(player_id, account_id, &character_name, respond_to),

            Command::EnterGame {
                player_id,
                session_id,
            } => self.enter_game(player_id, session_id).await,

            Command::GetCharacterByName { name, respond_to } => {
                let _ = respond_to.send(self.get_character_by_name(&name).await);
            }

            Command::GetFile {
                player_id,
                file_type,
                session_id,
                file_id,
                warp,
            } => {
                self.get_file(player_id, file_type, session_id, file_id, warp)
                    .await;
            }

            Command::GetMap { map_id, respond_to } => {
                let maps = self.maps.as_ref().expect("maps not loaded");
                match maps.get(&map_id) {
                    Some(map) => {
                        let _ = respond_to.send(Ok(map.to_owned()));
                    }
                    None => {
                        warn!("Map not found: {}", map_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Map".to_string(),
                            map_id,
                        ))));
                    }
                }
            }

            Command::GetNextPlayerId { respond_to } => {
                let _ = respond_to.send(self.get_next_player_id(300));
            }

            Command::GetPlayerParty {
                player_id,
                respond_to,
            } => {
                let _ = respond_to.send(self.get_player_party(player_id));
            }

            Command::GetPlayerCount { respond_to } => {
                let _ = respond_to.send(self.players.len());
            }

            Command::IsLoggedIn {
                account_id,
                respond_to,
            } => {
                let _ = respond_to.send(self.accounts.contains(&account_id));
            }

            Command::LoadMapFiles { world, respond_to } => {
                match load_maps(self.pool.to_owned(), world).await {
                    Ok(maps) => {
                        self.maps = Some(maps);
                        let _ = respond_to.send(());
                    }
                    Err(err) => {
                        warn!("Failed to load maps: {}", err);
                        let _ = respond_to.send(());
                    }
                }
            }

            Command::Login {
                player_id,
                name,
                password,
            } => self.login(player_id, name, password, world_handle),

            Command::PingPlayers => {
                for player in self.players.values() {
                    player.ping();
                }
            }

            Command::RemovePartyMember {
                player_id,
                target_player_id,
            } => self.remove_party_member(player_id, target_player_id),

            Command::ReportPlayer {
                player_id,
                reportee_name,
                message,
            } => self.report_player(player_id, reportee_name, message).await,

            Command::RequestAccountCreation { player_id, name } => {
                self.request_account_creation(player_id, name).await;
            }

            Command::RequestCharacterCreation { player_id } => {
                self.request_character_creation(player_id).await;
            }

            Command::RequestCharacterDeletion {
                player_id,
                character_id,
            } => {
                self.request_character_deletion(player_id, character_id)
                    .await
            }

            Command::RequestPartyList { player_id } => self.refresh_party(player_id).await,

            Command::Save => self.save().await,

            Command::SelectCharacter {
                player_id,
                character_id,
            } => self.select_character(player_id, character_id).await,

            Command::SendAdminMessage { player_id, message } => {
                self.send_admin_message(player_id, message).await
            }

            Command::SendPrivateMessage { from, to, message } => {
                self.send_private_message(&from, &to, &message).await
            }

            Command::Shutdown { respond_to } => self.shutdown(respond_to).await,

            Command::Tick => {
                self.tick().await;
            }

            Command::UpdatePartyHP {
                player_id,
                hp_percentage,
            } => self.update_party_hp(player_id, hp_percentage),

            Command::UpdatePartyExp {
                player_id,
                exp_gains,
            } => self.update_party_exp(player_id, exp_gains),

            Command::BanPlayer {
                victim_name,
                admin_name,
                duration,
                silent,
            } => {
                self.ban_player(victim_name, admin_name, duration, silent)
                    .await
            }

            Command::FindPlayer { player_id, name } => self.find_player(player_id, name),

            Command::FreePlayer { victim_name } => self.free_player(victim_name),

            Command::FreezePlayer {
                victim_name,
                admin_name,
            } => self.freeze_player(victim_name, admin_name),

            Command::JailPlayer {
                victim_name,
                admin_name,
            } => self.jail_player(victim_name, admin_name),

            Command::KickPlayer {
                victim_name,
                admin_name,
                silent,
            } => self.kick_player(victim_name, admin_name, silent),

            Command::MutePlayer {
                victim_name,
                admin_name,
            } => self.mute_player(victim_name, admin_name),

            Command::Quake { magnitude } => self.quake(magnitude),

            Command::RequestPlayerInfo {
                player_id,
                victim_name,
            } => self.request_player_info(player_id, victim_name),

            Command::RequestPlayerInventory {
                player_id,
                victim_name,
            } => self.request_player_inventory(player_id, victim_name),

            Command::ToggleGlobal { admin_name } => self.toggle_global(admin_name),

            Command::UnfreezePlayer {
                victim_name,
                admin_name,
            } => self.unfreeze_player(victim_name, admin_name),
            Command::RequestPlayerList { player_id } => self.request_player_list(player_id),
            Command::RequestPlayerNameList { player_id } => {
                self.request_player_name_list(player_id)
            }
        }
    }
}
