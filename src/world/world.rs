use crate::{player::PlayerHandle, SETTINGS};

use super::{account, Command, data};
use eo::data::{
    map::MapFile,
    pubs::{
        ClassFile, DropFile, InnFile, ItemFile, MasterFile, NPCFile, ShopFile, SpellFile, TalkFile,
    },
    EOShort,
};
use mysql_async::Pool;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc::UnboundedReceiver, Mutex},
    time,
};

#[derive(Debug)]
pub struct World {
    pub rx: UnboundedReceiver<Command>,
    players: Arc<Mutex<HashMap<EOShort, PlayerHandle>>>,
    accounts: Arc<Mutex<Vec<EOShort>>>,
    pool: Pool,
    maps: Option<Arc<Mutex<HashMap<EOShort, MapFile>>>>,
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
            Command::GetPlayerCount { respond_to } => {
                let players = players.lock().await;
                let _ = respond_to.send(players.len());
            }
            Command::GetNextPlayerId { respond_to } => {
                let players = players.lock().await;
                let _ = respond_to.send(get_next_player_id(&players, 1));
            }
            Command::AddPlayer {
                respond_to,
                player_id,
                player,
            } => {
                let mut players = players.lock().await;
                players.insert(player_id, player);
                let _ = respond_to.send(());
            }
            Command::DropPlayer {
                player_id,
                account_id,
                character_id: _,
                respond_to,
            } => {
                let mut players = players.lock().await;
                players.remove(&player_id).unwrap();

                if account_id > 0 {
                    let mut accounts = self.accounts.lock().await;
                    accounts.retain(|id| *id != account_id);
                }

                // TODO: unload character too
                let _ = respond_to.send(());
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
            Command::CreateAccount {
                details,
                register_ip,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::create_account(&mut conn, details, register_ip).await;
                let _ = respond_to.send(result);
            }
            Command::Login {
                name,
                password,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let mut accounts = self.accounts.lock().await;
                let result = account::login(&mut conn, &name, &password, &mut accounts).await;
                let _ = respond_to.send(result);
            }
            Command::RequestCharacterCreation {
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::request_character_creation(&mut conn, player).await;
                let _ = respond_to.send(result);
            }
            Command::CreateCharacter { details, player, respond_to } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::create_character(&mut conn, details, player).await;
                let _ = respond_to.send(result);
            },
            Command::RequestCharacterDeletion {
                character_id,
                player,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::request_character_deletion(&mut conn, character_id, player).await;
                let _ = respond_to.send(result);
            },
            Command::DeleteCharacter { session_id, character_id, player, respond_to } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let result = account::delete_character(&mut conn, session_id, character_id, player).await;
                let _ = respond_to.send(result);
            },
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
