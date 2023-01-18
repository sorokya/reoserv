use crate::{
    character::Character,
    errors::{
        CharacterNotFoundError, DataNotFoundError, MissingSessionIdError, WrongSessionIdError,
    },
    map::MapHandle,
    player::{PlayerHandle, ClientState},
    SETTINGS,
};

use super::{
    account::{self, calculate_stats, set_character_stat},
    chat::{
        broadcast_admin_message, broadcast_announcement, broadcast_global_message,
        broadcast_server_message, send_player_not_found, send_private_message,
    },
    data, enter_game, Command,
};
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable},
    protocol::{
        server::{
            init,
            welcome::{self, ReplySelectCharacter},
        },
        FileType, InitReply, ServerSettings, WelcomeReply, OnlinePlayers,
    },
    pubs::{
        DropFile, EcfFile, EifFile, EnfFile, EsfFile, InnFile, ShopFile, SkillMasterFile, TalkFile,
    },
};
use mysql_async::Pool;
use std::{collections::HashMap, convert::TryInto};
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
            } => {
                self.players.insert(player_id, player);
                let _ = respond_to.send(());
            }
            Command::BroadcastAdminMessage { name, message } => {
                broadcast_admin_message(&name, &message, &self.players).await;
            }
            Command::BroadcastAnnouncement { name, message } => {
                broadcast_announcement(&name, &message, &self.players).await;
            }
            Command::BroadcastGlobalMessage {
                target_player_id,
                name,
                message,
            } => {
                broadcast_global_message(target_player_id, &name, &message, &self.players).await;
            }
            Command::_BroadcastServerMessage { message } => {
                broadcast_server_message(&message, &self.players).await;
            }
            Command::CreateAccount {
                player,
                details,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::create_account(&mut conn, player, details).await;
                let _ = respond_to.send(result);
            }
            Command::CreateCharacter {
                details,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::create_character(&mut conn, details, player).await;
                let _ = respond_to.send(result);
            }
            Command::DeleteCharacter {
                player_id,
                character_id,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result =
                    account::delete_character(&mut conn, player_id, character_id, player).await;
                let _ = respond_to.send(result);
            }
            Command::DropPlayer {
                player_id,
                account_id,
                character_name,
                respond_to,
            } => {
                debug!(
                    "Dropping player! id: {}, account_id: {}, character_name: {}",
                    player_id, account_id, character_name
                );
                self.players.remove(&player_id).unwrap();

                if account_id > 0 {
                    self.accounts.retain(|id| *id != account_id);
                }

                if self.characters.contains_key(&character_name) {
                    self.characters.remove(&character_name);
                }

                let _ = respond_to.send(());
            }
            Command::EnterGame {
                session_id,
                player,
                respond_to,
            } => {
                match player.take_session_id().await {
                    Ok(actual_session_id) => {
                        if actual_session_id != session_id {
                            let _ = respond_to.send(Err(Box::new(WrongSessionIdError::new(
                                actual_session_id,
                                session_id,
                            ))));
                            return;
                        }
                        let map_id = match player.get_map_id().await {
                            Ok(map_id) => map_id,
                            Err(e) => {
                                let _ = respond_to.send(Err(Box::new(e)));
                                return;
                            }
                        };

                        if let Some(maps) = self.maps.as_ref() {
                            if let Some(map) = maps.get(&map_id) {
                                let player = player.to_owned();
                                let map = map.to_owned();
                                let _ = tokio::task::Builder::new().name("enter_game").spawn(
                                    async move {
                                        let result = enter_game(map, player).await;
                                        let _ = respond_to.send(result);
                                    },
                                );
                            } else {
                                // TODO: Move character to safe map
                                let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                                    "Map".to_string(),
                                    map_id,
                                ))));
                            }
                        } else {
                            let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                                "Map".to_string(),
                                map_id,
                            ))));
                        }
                    }
                    Err(e) => {
                        let _ = respond_to.send(Err(Box::new(e)));
                    }
                }
            }
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
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let (reply, account_id) =
                    match account::login(&mut conn, &name, &password, &mut self.accounts).await {
                        Ok((reply, account_id)) => (reply, account_id),
                        Err(err) => {
                            let _ = respond_to.send(Err(err));
                            return;
                        }
                    };
                player.set_account_id(account_id);
                player.set_state(ClientState::LoggedIn);
                let _ = respond_to.send(Ok(reply));
            }
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
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::request_account_creation(&mut conn, name, player).await;
                let _ = respond_to.send(result);
            }
            Command::RequestCharacterCreation { player, respond_to } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::request_character_creation(&mut conn, player).await;
                let _ = respond_to.send(result);
            }
            Command::RequestCharacterDeletion {
                character_id,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result =
                    account::request_character_deletion(&mut conn, character_id, player).await;
                let _ = respond_to.send(result);
            }
            Command::SelectCharacter {
                character_id,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let mut character = match account::select_character(
                    &mut conn,
                    character_id,
                    player.clone(),
                )
                .await
                {
                    Ok(character) => character,
                    Err(err) => {
                        let _ = respond_to.send(Err(err));
                        return;
                    }
                };

                let item_file = self.item_file.as_ref().expect("Item file not found");
                let class_file = self.class_file.as_ref().expect("Class file not found");
                calculate_stats(&mut character, item_file, class_file);

                let select_character = match self
                    .get_welcome_request_data(player.clone(), &character)
                    .await
                {
                    Ok(select_character) => select_character,
                    Err(err) => {
                        let _ = respond_to.send(Err(err));
                        return;
                    }
                };

                let player_id = player.get_player_id().await;
                self.characters
                    .insert(character.name.to_string(), player_id);
                player.set_character(Box::new(character));

                let _ = respond_to.send(Ok(welcome::Reply {
                    reply_code: WelcomeReply::SelectCharacter,
                    data: welcome::ReplyData::SelectCharacter(select_character),
                }));
            }
            Command::SendPrivateMessage { from, to, message } => {
                if let Ok(from_character) = from.get_character().await {
                    match self.get_character_by_name(&to).await {
                        Ok(character) => send_private_message(
                            &from_character.name,
                            character.player.as_ref().unwrap(),
                            &message,
                        ),
                        Err(_) => send_player_not_found(from, &to),
                    }
                }
            }
            Command::SetCharacterStat {
                target_name,
                stat_name,
                value,
            } => {
                if let Ok(mut character) = self.get_character_by_name(&target_name).await {
                    set_character_stat(&mut character, stat_name, value);

                    let item_file = self.item_file.as_ref().expect("Item file not found");
                    let class_file = self.class_file.as_ref().expect("Class file not found");
                    calculate_stats(&mut character, item_file, class_file);

                    // TODO: send packet telling player stats changed
                }
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

    async fn get_character_by_name(
        &self,
        name: &str,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(player_id) = self.characters.get(name) {
            if let Some(player) = self.players.get(player_id) {
                // Safe to assume this will work if we got this far
                let character = player.get_character().await.unwrap();
                Ok(character)
            } else {
                Err(Box::new(DataNotFoundError::new(
                    "Player".to_string(),
                    *player_id,
                )))
            }
        } else {
            Err(Box::new(CharacterNotFoundError::new(name.to_string())))
        }
    }

    async fn get_welcome_request_data(
        &self,
        player: PlayerHandle,
        character: &Character,
    ) -> Result<ReplySelectCharacter, Box<dyn std::error::Error + Send + Sync>> {
        let (map_rid, map_filesize) = {
            let maps = self.maps.as_ref().expect("Maps not loaded");
            let map = match maps.get(&character.map_id) {
                Some(map) => map,
                None => {
                    error!("Map not found: {}", character.map_id);
                    return Err(Box::new(DataNotFoundError::new(
                        "Map".to_string(),
                        character.map_id,
                    )));
                }
            };
            map.get_rid_and_size().await
        };

        let (eif_rid, eif_length) = {
            let item_file = self.item_file.as_ref().expect("Item file not loaded");
            (item_file.rid, item_file.num_items)
        };

        let (ecf_rid, ecf_length) = {
            let class_file = self.class_file.as_ref().expect("Class file not loaded");
            (class_file.rid, class_file.num_classes)
        };

        let (enf_rid, enf_length) = {
            let npc_file = self.npc_file.as_ref().expect("NPC file not loaded");
            (npc_file.rid, npc_file.num_npcs)
        };

        let (esf_rid, esf_length) = {
            let spell_file = self.spell_file.as_ref().expect("Spell file not loaded");
            (spell_file.rid, spell_file.num_spells)
        };

        let settings = ServerSettings {
            jail_map: SETTINGS.jail.map.try_into().expect("Invalid map id"),
            rescue_map: 4,
            rescue_x: 24,
            rescue_y: 24,
            light_guide_flood_rate: 10,
            guardian_flood_rate: 10,
            gm_flood_rate: 10,
            hgm_flood_rate: 0,
        };

        let session_id = player.generate_session_id().await;

        Ok(ReplySelectCharacter {
            session_id,
            character_id: character.id,
            map_id: character.map_id,
            map_rid,
            map_filesize,
            eif_rid,
            eif_length,
            enf_rid,
            enf_length,
            esf_rid,
            esf_length,
            ecf_rid,
            ecf_length,
            name: character.name.to_string(),
            title: character.title.clone().unwrap_or_default(),
            guild_name: character.guild_name.clone().unwrap_or_default(),
            guild_rank_name: character.guild_rank_string.clone().unwrap_or_default(),
            class_id: character.class,
            guild_tag: character.guild_tag.clone().unwrap_or_default(),
            admin: character.admin_level,
            level: character.level,
            experience: character.experience,
            usage: character.usage,
            stats: character.get_character_stats_2(),
            paperdoll: character.paperdoll.to_owned(),
            guild_rank: character.guild_rank_id.unwrap_or_default(),
            settings,
            login_message: match character.usage {
                0 => 2,
                _ => 0,
            },
        })
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
