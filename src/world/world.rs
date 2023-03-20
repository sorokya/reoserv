use crate::{
    errors::{DataNotFoundError, MissingSessionIdError, WrongSessionIdError},
    map::MapHandle,
    player::PlayerHandle,
};

use super::{data, Command};
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable},
    protocol::{server::init, FileType, InitReply, OnlinePlayers},
    pubs::{
        DropFile, EcfFile, EifFile, EnfFile, EsfFile, InnFile, ShopFile, SkillMasterFile, TalkFile,
    },
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
    class_file: Option<EcfFile>,
    drop_file: Option<DropFile>,
    inn_file: Option<InnFile>,
    item_file: Option<EifFile>,
    master_file: Option<SkillMasterFile>,
    npc_file: Option<EnfFile>,
    shop_file: Option<ShopFile>,
    spell_file: Option<EsfFile>,
    talk_file: Option<TalkFile>,
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
            class_file: None,
            drop_file: None,
            inn_file: None,
            item_file: None,
            master_file: None,
            npc_file: None,
            shop_file: None,
            spell_file: None,
            talk_file: None,
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
            Command::_GetClass {
                class_id,
                respond_to,
            } => {
                let class_file = self.class_file.as_ref().expect("classes not loaded");
                match class_file.classes.get(class_id as usize - 1) {
                    Some(class) => {
                        let _ = respond_to.send(Ok(class.clone()));
                    }
                    None => {
                        warn!("Class not found: {}", class_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Class".to_string(),
                            class_id as EOShort,
                        ))));
                    }
                }
            }
            Command::GetDropRecord { npc_id, respond_to } => {
                let drops = self.drop_file.as_ref().expect("drops not loaded");
                match drops.npcs.iter().find(|d| d.npc_id == npc_id) {
                    Some(drop) => {
                        let _ = respond_to.send(Some(drop.clone()));
                    }
                    None => {
                        let _ = respond_to.send(None);
                    }
                }
            }
            Command::_GetItem {
                item_id,
                respond_to,
            } => {
                let item_file = self.item_file.as_ref().expect("classes not loaded");
                match item_file.items.get(item_id as usize - 1) {
                    Some(item) => {
                        let _ = respond_to.send(Ok(item.clone()));
                    }
                    None => {
                        warn!("Item not found: {}", item_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Item".to_string(),
                            item_id,
                        ))));
                    }
                }
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
            Command::GetNpc { npc_id, respond_to } => {
                let npcs = self.npc_file.as_ref().expect("npcs not loaded");
                match npcs.npcs.get(npc_id as usize - 1) {
                    Some(npc) => {
                        let _ = respond_to.send(Ok(npc.clone()));
                    }
                    None => {
                        warn!("NPC not found: {}", npc_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "NPC".to_string(),
                            npc_id,
                        ))));
                    }
                }
            }
            Command::GetOnlineList { respond_to } => {
                let _ = respond_to.send(self.get_online_list().await);
            }
            Command::GetPlayerCount { respond_to } => {
                let _ = respond_to.send(self.players.len());
            }
            Command::GetTalkRecord { npc_id, respond_to } => {
                let talks = self.talk_file.as_ref().expect("talks not loaded");
                match talks.npcs.iter().find(|t| t.npc_id == npc_id) {
                    Some(talk) => {
                        let _ = respond_to.send(Some(talk.clone()));
                    }
                    None => {
                        let _ = respond_to.send(None);
                    }
                }
            }
            Command::LoadMapFiles {
                world_handle,
                respond_to,
            } => match data::load_maps(world_handle).await {
                Ok(maps) => {
                    self.maps = Some(maps);
                    let _ = respond_to.send(());
                }
                Err(err) => {
                    warn!("Failed to load maps: {}", err);
                    let _ = respond_to.send(());
                }
            },
            Command::LoadPubFiles { respond_to } => {
                let (
                    class_file,
                    drop_file,
                    inn_file,
                    item_file,
                    master_file,
                    npc_file,
                    shop_file,
                    spell_file,
                    talk_file,
                ) = tokio::join!(
                    data::load_class_file("pub/dat001.ecf".to_string()),
                    data::load_drop_file("pub/dtd001.edf".to_string()),
                    data::load_inn_file("pub/din001.eid".to_string()),
                    data::load_item_file("pub/dat001.eif".to_string()),
                    data::load_master_file("pub/dsm001.emf".to_string()),
                    data::load_npc_file("pub/dtn001.enf".to_string()),
                    data::load_shop_file("pub/dts001.esf".to_string()),
                    data::load_spell_file("pub/dsl001.esf".to_string()),
                    data::load_talk_file("pub/ttd001.etf".to_string()),
                );
                // TODO: allow not having all of these
                self.class_file = Some(class_file.unwrap());
                self.drop_file = Some(drop_file.unwrap());
                self.inn_file = Some(inn_file.unwrap());
                self.item_file = Some(item_file.unwrap());
                self.master_file = Some(master_file.unwrap());
                self.npc_file = Some(npc_file.unwrap());
                self.shop_file = Some(shop_file.unwrap());
                self.spell_file = Some(spell_file.unwrap());
                self.talk_file = Some(talk_file.unwrap());
                let _ = respond_to.send(());
            }
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
                    let mut reply = init::Init::default();
                    let item_file = self.item_file.as_ref().expect("Item file not loaded");
                    reply.reply_code = InitReply::FileEif;
                    reply.data = init::InitData::FileEif(init::InitFileEif {
                        file_id: 1, // TODO: Pub splitting
                        content: item_file.serialize(),
                    });
                    Ok(reply)
                }
                FileType::Npc => {
                    let mut reply = init::Init::default();
                    let npc_file = self.npc_file.as_ref().expect("NPC file not loaded");
                    reply.reply_code = InitReply::FileEnf;
                    reply.data = init::InitData::FileEnf(init::InitFileEnf {
                        file_id: 1, // TODO: Pub splitting
                        content: npc_file.serialize(),
                    });
                    Ok(reply)
                }
                FileType::Spell => {
                    let mut reply = init::Init::default();
                    let spell_file = self.spell_file.as_ref().expect("Spell file not loaded");
                    reply.reply_code = InitReply::FileEsf;
                    reply.data = init::InitData::FileEsf(init::InitFileEsf {
                        file_id: 1, // TODO: Pub splitting
                        content: spell_file.serialize(),
                    });
                    Ok(reply)
                }
                FileType::Class => {
                    let mut reply = init::Init::default();
                    let class_file = self.class_file.as_ref().expect("Class file not loaded");
                    reply.reply_code = InitReply::FileEcf;
                    reply.data = init::InitData::FileEcf(init::InitFileEcf {
                        file_id: 1, // TODO: Pub splitting
                        content: class_file.serialize(),
                    });
                    Ok(reply)
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
