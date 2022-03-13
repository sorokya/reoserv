use crate::{
    character::Character,
    map::MapHandle,
    player::{PlayerHandle, State},
    world::DataNotFoundError,
    SETTINGS,
};

use super::{account, data, enter_game, Command};
use eo::{
    data::{
        pubs::{
            ClassFile, DropFile, InnFile, ItemFile, MasterFile, NPCFile, ShopFile, SpellFile,
            TalkFile,
        },
        EOInt, EOShort, Serializeable,
    },
    net::{
        packets::server::{
            init,
            welcome::{self, SelectCharacter},
        },
        replies::{InitReply, WelcomeReply},
        FileType, ServerSettings,
    },
};
use mysql_async::Pool;
use std::{collections::HashMap, convert::TryInto, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc::UnboundedReceiver, Mutex},
    time,
};

#[derive(Debug)]
pub struct World {
    pub rx: UnboundedReceiver<Command>,
    players: Arc<Mutex<HashMap<EOShort, PlayerHandle>>>,
    accounts: Arc<Mutex<Vec<EOInt>>>,
    pool: Pool,
    maps: Option<Arc<Mutex<HashMap<EOShort, MapHandle>>>>,
    class_file: Option<Arc<Mutex<ClassFile>>>,
    drop_file: Option<Arc<Mutex<DropFile>>>,
    inn_file: Option<Arc<Mutex<InnFile>>>,
    item_file: Option<Arc<Mutex<ItemFile>>>,
    master_file: Option<Arc<Mutex<MasterFile>>>,
    npc_file: Option<Arc<Mutex<NPCFile>>>,
    shop_file: Option<Arc<Mutex<ShopFile>>>,
    spell_file: Option<Arc<Mutex<SpellFile>>>,
    talk_file: Option<Arc<Mutex<TalkFile>>>,
}

impl World {
    pub fn new(rx: UnboundedReceiver<Command>, pool: Pool) -> Self {
        Self {
            rx,
            pool,
            players: Arc::new(Mutex::new(HashMap::new())),
            accounts: Arc::new(Mutex::new(Vec::new())),
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
        let players = self.players.clone();

        match command {
            Command::AddPlayer {
                respond_to,
                player_id,
                player,
            } => {
                let mut players = players.lock().await;
                players.insert(player_id, player);
                let _ = respond_to.send(());
            }
            Command::CreateAccount {
                details,
                register_ip,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::create_account(&mut conn, details, register_ip).await;
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
                session_id,
                character_id,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result =
                    account::delete_character(&mut conn, session_id, character_id, player).await;
                let _ = respond_to.send(result);
            }
            Command::DropPlayer {
                player_id,
                account_id,
                respond_to,
            } => {
                let mut players = players.lock().await;
                players.remove(&player_id).unwrap();

                if account_id > 0 {
                    let mut accounts = self.accounts.lock().await;
                    accounts.retain(|id| *id != account_id);
                }

                let _ = respond_to.send(());
            }
            Command::EnterGame { player, respond_to } => {
                let map_id = match player.get_map_id().await {
                    Ok(map_id) => map_id,
                    Err(e) => {
                        error!("Couldn't get map id: {}", e);
                        return;
                    }
                };

                let maps = self.maps.as_ref().expect("maps not loaded");
                let maps = maps.lock().await;
                let map = match maps.get(&map_id) {
                    Some(map) => map.to_owned(),
                    None => {
                        error!("Map not found: {}", map_id);
                        // TODO: warp player to valid position
                        return;
                    }
                };
                tokio::task::Builder::new()
                    .name("enter_game")
                    .spawn(async move {
                        let result = enter_game(map, player).await;
                        let _ = respond_to.send(result);
                    });
            }
            Command::GetClass {
                class_id,
                respond_to,
            } => {
                let classes = self.class_file.as_ref().expect("classes not loaded");
                let classes = classes.lock().await;
                match classes.records.iter().find(|c| c.id == class_id as EOInt) {
                    Some(class) => {
                        let _ = respond_to.send(Ok(class.clone()));
                    }
                    None => {
                        error!("Class not found: {}", class_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Class".to_string(),
                            class_id as EOShort,
                        ))));
                    }
                }
            }
            Command::GetItem {
                item_id,
                respond_to,
            } => {
                let item_file = self.item_file.as_ref().expect("classes not loaded");
                let item_file = item_file.lock().await;
                match item_file.records.iter().find(|i| i.id == item_id as EOInt) {
                    Some(item) => {
                        let _ = respond_to.send(Ok(item.clone()));
                    }
                    None => {
                        error!("Item not found: {}", item_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Item".to_string(),
                            item_id,
                        ))));
                    }
                }
            }
            Command::GetFile {
                file_type,
                player,
                respond_to,
            } => {
                let result = self.get_file(file_type, player).await;
                let _ = respond_to.send(result);
            }
            Command::GetNextPlayerId { respond_to } => {
                let players = players.lock().await;
                let _ = respond_to.send(get_next_player_id(&players, 1));
            }
            Command::GetPlayerCount { respond_to } => {
                let players = players.lock().await;
                let _ = respond_to.send(players.len());
            }
            Command::LoadMapFiles { respond_to } => match data::load_maps().await {
                Ok(maps) => {
                    self.maps = Some(Arc::new(Mutex::new(maps)));
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
                self.class_file = Some(Arc::new(Mutex::new(class_file.unwrap())));
                self.drop_file = Some(Arc::new(Mutex::new(drop_file.unwrap())));
                self.inn_file = Some(Arc::new(Mutex::new(inn_file.unwrap())));
                self.item_file = Some(Arc::new(Mutex::new(item_file.unwrap())));
                self.master_file = Some(Arc::new(Mutex::new(master_file.unwrap())));
                self.npc_file = Some(Arc::new(Mutex::new(npc_file.unwrap())));
                self.shop_file = Some(Arc::new(Mutex::new(shop_file.unwrap())));
                self.spell_file = Some(Arc::new(Mutex::new(spell_file.unwrap())));
                self.talk_file = Some(Arc::new(Mutex::new(talk_file.unwrap())));
                let _ = respond_to.send(());
            }
            Command::Login {
                name,
                password,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let mut accounts = self.accounts.lock().await;
                let (reply, account_id) =
                    match account::login(&mut conn, &name, &password, &mut accounts).await {
                        Ok((reply, account_id)) => (reply, account_id),
                        Err(err) => {
                            let _ = respond_to.send(Err(err));
                            return;
                        }
                    };
                player.set_account_id(account_id);
                player.set_state(State::LoggedIn);
                let _ = respond_to.send(Ok(reply));
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
                let character = match account::select_character(
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

                player.set_character(character);

                let _ = respond_to.send(Ok(welcome::Reply {
                    reply: WelcomeReply::SelectCharacter,
                    select_character: Some(select_character),
                    enter_game: None,
                }));
            }
            Command::StartPingTimer { respond_to } => {
                let mut ping_interval =
                    time::interval(Duration::from_secs(SETTINGS.server.ping_rate.into()));
                let ping_players = players.clone();
                // Skip the first tick because it's instant
                ping_interval.tick().await;
                tokio::spawn(async move {
                    loop {
                        ping_interval.tick().await;
                        let players = ping_players.lock().await;
                        for (_, player) in players.iter() {
                            player.ping();
                        }
                    }
                });
                let _ = respond_to.send(());
            }
        }
    }

    async fn get_welcome_request_data(
        &self,
        player: PlayerHandle,
        character: &Character,
    ) -> Result<SelectCharacter, Box<dyn std::error::Error + Send + Sync>> {
        let player_id = player.get_player_id().await;
        let (map_rid, map_filesize) = {
            let maps = self.maps.as_ref().expect("Maps not loaded");
            let maps = maps.lock().await;
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
            let item_file = item_file.lock().await;
            (item_file.rid, item_file.len())
        };

        let (ecf_rid, ecf_length) = {
            let class_file = self.class_file.as_ref().expect("Class file not loaded");
            let class_file = class_file.lock().await;
            (class_file.rid, class_file.len())
        };

        let (enf_rid, enf_length) = {
            let npc_file = self.npc_file.as_ref().expect("NPC file not loaded");
            let npc_file = npc_file.lock().await;
            (npc_file.rid, npc_file.len())
        };

        let (esf_rid, esf_length) = {
            let spell_file = self.spell_file.as_ref().expect("Spell file not loaded");
            let spell_file = spell_file.lock().await;
            (spell_file.rid, spell_file.len())
        };

        let settings = ServerSettings {
            jail_map_id: SETTINGS.jail.map.try_into().expect("Invalid map id"),
            unknown_1: 4,
            unknown_2: 24,
            unknown_3: 24,
            light_guide_flood_rate: 10,
            guardian_flood_rate: 10,
            game_master_flood_rate: 10,
            unknown_4: 2,
        };

        Ok(SelectCharacter {
            player_id,
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
            admin_level: character.admin_level,
            level: character.level,
            experience: character.experience,
            usage: character.usage,
            stats: character.get_character_stats_2(),
            paperdoll: character.paperdoll,
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
        player: PlayerHandle,
    ) -> Result<init::Reply, Box<dyn std::error::Error + Send + Sync>> {
        match file_type {
            FileType::Map => {
                let map_id = match player.get_map_id().await {
                    Ok(map_id) => map_id,
                    Err(e) => {
                        warn!("Player requested map with no character selected");
                        return Err(Box::new(e));
                    }
                };

                let mut reply = init::ReplyFileMap::new();
                let maps = self.maps.as_ref().expect("Maps not loaded");
                let maps = maps.lock().await;
                let map = match maps.get(&map_id) {
                    Some(map) => map,
                    None => {
                        error!("Requested map not found: {}", map_id);
                        return Err(Box::new(DataNotFoundError::new("Map".to_string(), map_id)));
                    }
                };
                reply.data = map.serialize().await;
                Ok(init::Reply {
                    reply_code: InitReply::FileMap,
                    reply: Box::new(reply),
                })
            }
            FileType::Item => {
                let mut reply = init::ReplyFileItem::new();
                let item_file = self.item_file.as_ref().expect("Item file not loaded");
                let item_file = item_file.lock().await;
                reply.id = 1;
                reply.data = item_file.serialize();
                Ok(init::Reply {
                    reply_code: InitReply::FileItem,
                    reply: Box::new(reply),
                })
            }
            FileType::NPC => {
                let mut reply = init::ReplyFileNPC::new();
                let npc_file = self.npc_file.as_ref().expect("NPC file not loaded");
                let npc_file = npc_file.lock().await;
                reply.id = 1;
                reply.data = npc_file.serialize();
                Ok(init::Reply {
                    reply_code: InitReply::FileNPC,
                    reply: Box::new(reply),
                })
            }
            FileType::Spell => {
                let mut reply = init::ReplyFileSpell::new();
                let spell_file = self.spell_file.as_ref().expect("Spell file not loaded");
                let spell_file = spell_file.lock().await;
                reply.id = 1;
                reply.data = spell_file.serialize();
                Ok(init::Reply {
                    reply_code: InitReply::FileSpell,
                    reply: Box::new(reply),
                })
            }
            FileType::Class => {
                let mut reply = init::ReplyFileClass::new();
                let class_file = self.class_file.as_ref().expect("Class file not loaded");
                let class_file = class_file.lock().await;
                reply.id = 1;
                reply.data = class_file.serialize();
                Ok(init::Reply {
                    reply_code: InitReply::FileClass,
                    reply: Box::new(reply),
                })
            }
        }
    }
}

fn get_next_player_id(
    players: &tokio::sync::MutexGuard<HashMap<EOShort, PlayerHandle>>,
    seed: EOShort,
) -> EOShort {
    let id = seed;
    for player_id in players.iter().map(|c| *c.0) {
        if player_id == id {
            return get_next_player_id(players, id + 1);
        }
    }
    id
}
