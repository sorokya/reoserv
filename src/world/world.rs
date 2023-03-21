use crate::{
    errors::{DataNotFoundError, MissingSessionIdError, WrongSessionIdError},
    map::MapHandle,
    player::PlayerHandle,
    CLASS_DB, ITEM_DB, NPC_DB, SPELL_DB,
};

use super::{load_maps::load_maps, Command};
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable},
    protocol::{server::init, FileType, InitReply, OnlinePlayers},
};
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
}

mod account;
mod add_player;
mod chat;
mod drop_player;
mod enter_game;
mod get_character_by_name;
mod get_welcome_request_data;

impl World {
    pub fn new(rx: UnboundedReceiver<Command>, pool: Pool) -> Self {
        Self {
            rx,
            pool,
            players: HashMap::new(),
            accounts: Vec::new(),
            characters: HashMap::new(),
            maps: None,
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
                let _ = respond_to.send(get_next_player_id(&self.players, 300));
            }
            Command::GetOnlineList { respond_to } => {
                let _ = respond_to.send(self.get_online_list().await);
            }
            Command::GetPlayerCount { respond_to } => {
                let _ = respond_to.send(self.players.len());
            }
            Command::LoadMapFiles { respond_to } => match load_maps().await {
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
            Command::SendPrivateMessage { from, to, message } => {
                self.send_private_message(&from, &to, &message).await
            }
            Command::SpawnNpcs => {
                for map in self.maps.as_ref().unwrap().values() {
                    map.spawn_npcs();
                }
            }
            Command::ActNpcs => {
                for map in self.maps.as_ref().unwrap().values() {
                    map.act_npcs();
                }
            }
        }
    }

    async fn get_file(
        &self,
        file_type: FileType,
        session_id: EOShort,
        _file_id: Option<EOChar>,
        player: PlayerHandle,
    ) -> Result<init::Init, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(actual_session_id) = player.get_session_id().await {
            if actual_session_id != session_id {
                return Err(Box::new(WrongSessionIdError::new(
                    actual_session_id,
                    session_id,
                )));
            }

            match file_type {
                FileType::Map => {
                    let map_id = match player.get_map_id().await {
                        Ok(map_id) => map_id,
                        Err(e) => {
                            warn!("Player requested map with no character selected");
                            return Err(Box::new(e));
                        }
                    };

                    let mut reply = init::Init::default();
                    let maps = self.maps.as_ref().expect("Maps not loaded");
                    let map = match maps.get(&map_id) {
                        Some(map) => map,
                        None => {
                            error!("Requested map not found: {}", map_id);
                            return Err(Box::new(DataNotFoundError::new(
                                "Map".to_string(),
                                map_id,
                            )));
                        }
                    };
                    reply.reply_code = InitReply::FileEmf;
                    reply.data = init::InitData::FileEmf(init::InitFileEmf {
                        content: map.serialize().await,
                    });
                    Ok(reply)
                }
                FileType::Item => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEif,
                        data: init::InitData::FileEif(init::InitFileEif {
                            file_id: 1, // TODO: Pub splitting
                            content: ITEM_DB.serialize(),
                        }),
                    })
                }
                FileType::Npc => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEnf,
                        data: init::InitData::FileEnf(init::InitFileEnf {
                            file_id: 1, // TODO: Pub splitting
                            content: NPC_DB.serialize(),
                        }),
                    })
                }
                FileType::Spell => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEsf,
                        data: init::InitData::FileEsf(init::InitFileEsf {
                            file_id: 1, // TODO: Pub splitting
                            content: SPELL_DB.serialize(),
                        }),
                    })
                }
                FileType::Class => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEcf,
                        data: init::InitData::FileEcf(init::InitFileEcf {
                            file_id: 1, // TODO: Pub splitting
                            content: CLASS_DB.serialize(),
                        }),
                    })
                }
            }
        } else {
            Err(Box::new(MissingSessionIdError))
        }
    }

    async fn get_online_list(&self) -> Vec<OnlinePlayers> {
        let mut online_list = Vec::new();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                let mut entry = OnlinePlayers::new();
                entry.name = character.name.to_string();
                entry.class_id = character.class;
                entry.guild_tag = character.guild_tag.clone().unwrap_or_default();
                entry.title = character.title.clone().unwrap_or_default();
                entry.icon = character.get_icon();
                online_list.push(entry);
            }
        }
        online_list
    }
}

fn get_next_player_id(players: &HashMap<EOShort, PlayerHandle>, seed: EOShort) -> EOShort {
    if players.iter().any(|(id, _)| *id == seed) {
        get_next_player_id(players, seed + 1)
    } else {
        seed
    }
}
