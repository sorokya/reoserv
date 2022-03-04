use crate::{player::PlayerHandle, settings::Settings, world::get_account::get_account};

use super::{data, Command, create_account::create_account};
use eo::data::{
    map::MapFile,
    pubs::{
        ClassFile, DropFile, InnFile, ItemFile, MasterFile, NPCFile, ShopFile, SpellFile, TalkFile,
    },
    EOShort,
};
use lazy_static::lazy_static;
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
        lazy_static! {
            static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
        };

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
                        let mut players = ping_players.lock().await;
                        for (_, player) in players.iter_mut() {
                            if let Err(e) = player.ping() {
                                let _ = player.close(format!("Unknown error: {}", e));
                            }
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
                respond_to,
            } => {
                let mut players = players.lock().await;
                players.remove(&player_id).unwrap();
                // TODO: unload account/character too
                let _ = respond_to.send(());
            }
            Command::AccountNameInUse {
                name,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                let account = match get_account(&mut conn, &name).await {
                    Ok(account) => account,
                    Err(e) => {
                        error!("Failed to get account: {}", e);
                        let _ = respond_to.send(Err(e));
                        return;
                    }
                };
                let _ = respond_to.send(Ok(account.is_some()));
            }
            Command::ValidateName {
                name: _,
                respond_to,
            } => {
                // TODO validate name
                let _ = respond_to.send(true);
            }
            Command::CreateAccount {
                name,
                password_hash,
                real_name,
                location,
                email,
                computer,
                hdid,
                register_ip,
                respond_to,
            } => {
                let mut conn = self.pool.get_conn().await.unwrap();
                match create_account(&mut conn, name, password_hash, real_name, location, email, computer, hdid, register_ip).await {
                    Ok(_) => respond_to.send(Ok(())).unwrap(),
                    Err(e) => respond_to.send(Err(e)).unwrap(),
                }
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
