use crate::{errors::DataNotFoundError, map::MapHandle, player::PlayerHandle};

use super::{load_maps::load_maps, Command};
use eo::data::{EOInt, EOShort};
use mysql_async::Pool;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct World {
    pub rx: UnboundedReceiver<Command>,
    players: HashMap<EOShort, PlayerHandle>,
    accounts: Vec<EOInt>,
    characters: HashMap<String, EOShort>,
    pool: Pool,
    maps: Option<HashMap<EOShort, MapHandle>>,
    npc_spawn_ticks: EOInt,
    item_spawn_ticks: EOInt,
    player_recover_ticks: EOInt,
    npc_recover_ticks: EOInt,
    quake_ticks: EOInt,
    spike_ticks: EOInt,
}

mod account;
mod add_player;
mod admin;
mod chat;
mod drop_player;
mod enter_game;
mod get_character_by_name;
mod get_file;
mod get_next_player_id;
mod get_online_list;
mod get_welcome_request_data;
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
            npc_spawn_ticks: 0,
            item_spawn_ticks: 0,
            player_recover_ticks: 0,
            npc_recover_ticks: 0,
            quake_ticks: 0,
            spike_ticks: 0,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
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

            Command::_BroadcastServerMessage { message } => {
                self.broadcast_server_message(&message).await
            }

            Command::CreateAccount {
                player,
                details,
                respond_to,
            } => self.create_account(player, details, respond_to).await,

            Command::CreateCharacter {
                details,
                player,
                respond_to,
            } => self.create_character(player, details, respond_to).await,

            Command::DeleteCharacter {
                session_id,
                character_id,
                player,
                respond_to,
            } => {
                self.delete_character(player, session_id, character_id, respond_to)
                    .await
            }

            Command::DropPlayer {
                player_id,
                account_id,
                character_name,
                respond_to,
            } => self.drop_player(player_id, account_id, &character_name, respond_to),

            Command::EnterGame {
                session_id,
                player,
                respond_to,
            } => self.enter_game(player, session_id, respond_to).await,

            Command::GetCharacterByName { name, respond_to } => {
                let _ = respond_to.send(self.get_character_by_name(&name).await);
            }

            Command::GetFile {
                file_type,
                session_id,
                file_id,
                player,
                respond_to,
            } => {
                let result = self.get_file(file_type, session_id, file_id, player).await;
                let _ = respond_to.send(result);
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

            Command::GetOnlineList { respond_to } => {
                let _ = respond_to.send(self.get_online_list().await);
            }

            Command::GetPlayerCount { respond_to } => {
                let _ = respond_to.send(self.players.len());
            }

            Command::LoadMapFiles { respond_to } => match load_maps(self.pool.to_owned()).await {
                Ok(maps) => {
                    self.maps = Some(maps);
                    let _ = respond_to.send(());
                }
                Err(err) => {
                    warn!("Failed to load maps: {}", err);
                    let _ = respond_to.send(());
                }
            },

            Command::Login {
                name,
                password,
                player,
                respond_to,
            } => self.login(player, &name, &password, respond_to).await,

            Command::PingPlayers => {
                for player in self.players.values() {
                    player.ping();
                }
            }

            Command::ReportPlayer {
                player_id,
                reportee_name,
                message,
            } => self.report_player(player_id, reportee_name, message).await,

            Command::RequestAccountCreation {
                name,
                player,
                respond_to,
            } => {
                self.request_account_creation(player, name, respond_to)
                    .await
            }

            Command::RequestCharacterCreation { player, respond_to } => {
                self.request_character_creation(player, respond_to).await
            }

            Command::RequestCharacterDeletion {
                character_id,
                player,
                respond_to,
            } => {
                self.request_character_deletion(player, character_id, respond_to)
                    .await
            }

            Command::SelectCharacter {
                character_id,
                player,
                respond_to,
            } => {
                self.select_character(player, character_id, respond_to)
                    .await
            }

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
        }
    }
}
