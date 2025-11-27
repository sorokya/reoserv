use std::path::Path;

use chrono::{NaiveDateTime, TimeZone, Utc};
use eolib::protocol::{Coords, Direction};

use crate::map::{chest::ChestItem, npc::NpcOpponent, Item};

use super::super::Map;

impl Map {
    pub async fn save(&mut self) {
        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get connection from pool: {}", e);
                return;
            }
        };

        let now = chrono::Utc::now();

        for character in self.characters.values_mut() {
            if let Some(logged_in_at) = character.logged_in_at {
                character.usage += (now.timestamp() - logged_in_at.timestamp()) as i32 / 60;
            }

            if let Err(e) = character.save(&mut conn).await {
                error!("Failed to update character: {}", e);
                continue;
            }
        }

        if self.id == 0 {
            return;
        }

        // Check if data/map_saves directory exists
        let save_dir = Path::new("data/map_saves");
        match save_dir.try_exists() {
            Ok(true) => {}
            Ok(false) => {
                if let Err(e) = tokio::fs::create_dir_all(save_dir).await {
                    error!("Failed to create map_saves directory: {}", e);
                    return;
                }
            }
            Err(e) => {
                error!("Failed to check if map_saves directory exists: {}", e);
                return;
            }
        }

        let save_data = MapSaveData {
            rid: self.file.rid,
            items: self
                .items
                .iter()
                .map(|(index, item)| SavedItem {
                    index: *index,
                    id: item.id,
                    amount: item.amount,
                    x: item.coords.x,
                    y: item.coords.y,
                    owner: item.owner,
                    ticks: item.protected_ticks,
                })
                .collect(),
            npcs: self
                .npcs
                .iter()
                .map(|(index, npc)| SavedNpc {
                    index: *index,
                    x: npc.coords.x,
                    y: npc.coords.y,
                    direction: i32::from(npc.direction),
                    hp: npc.hp,
                    alive: npc.alive,
                    ticks: npc.spawn_ticks,
                    opponents: npc
                        .opponents
                        .iter()
                        .map(|opp| SavedNpcOpponent {
                            id: opp.player_id,
                            damage: opp.damage_dealt,
                            ticks: opp.bored_ticks,
                        })
                        .collect(),
                })
                .collect(),
            chest_items: self
                .chests
                .iter()
                .flat_map(|chest| {
                    chest.items.iter().map(|item| SavedChestItem {
                        x: chest.coords.x,
                        y: chest.coords.y,
                        slot: item.slot,
                        id: item.item_id,
                        amount: item.amount,
                    })
                })
                .collect(),
            chest_spawns: self
                .chests
                .iter()
                .flat_map(|chest| {
                    chest.spawns.iter().map(|spawn| SavedChestSpawn {
                        x: chest.coords.x,
                        y: chest.coords.y,
                        slot: spawn.slot,
                        taken: spawn.last_taken.naive_utc(),
                    })
                })
                .collect(),
        };

        let save_path = save_dir.join(format!("{:05}.json", self.id));
        if let Err(e) = tokio::fs::write(
            save_path,
            serde_json::to_string(&save_data).unwrap_or_default(),
        )
        .await
        {
            error!("Failed to write map save file: {}", e);
        }
    }

    pub async fn load(&mut self) {
        let save_path = Path::new("data/map_saves").join(format!("{:05}.json", self.id));
        let data = match tokio::fs::read_to_string(save_path).await {
            Ok(data) => data,
            Err(_) => return, // No save file, nothing to load
        };

        let save_data: MapSaveData = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to deserialize map save data: {}", e);
                return;
            }
        };

        if save_data.rid != self.file.rid {
            return;
        }

        self.spawn_npcs();

        for saved_item in save_data.items {
            self.items.insert(
                saved_item.index,
                Item {
                    id: saved_item.id,
                    amount: saved_item.amount,
                    coords: Coords {
                        x: saved_item.x,
                        y: saved_item.y,
                    },
                    owner: saved_item.owner,
                    protected_ticks: saved_item.ticks,
                },
            );
        }

        for saved_npc in save_data.npcs {
            if let Some(npc) = self.npcs.get_mut(&saved_npc.index) {
                npc.coords = Coords {
                    x: saved_npc.x,
                    y: saved_npc.y,
                };
                npc.direction = Direction::from(saved_npc.direction);
                npc.hp = saved_npc.hp;
                npc.alive = saved_npc.alive;
                npc.spawn_ticks = saved_npc.ticks;
                npc.opponents = saved_npc
                    .opponents
                    .into_iter()
                    .map(|opp| NpcOpponent {
                        player_id: opp.id,
                        damage_dealt: opp.damage,
                        bored_ticks: opp.ticks,
                    })
                    .collect();
            }
        }

        for saved_chest_item in save_data.chest_items {
            if let Some(chest) = self.chests.iter_mut().find(|chest| {
                chest.coords.x == saved_chest_item.x && chest.coords.y == saved_chest_item.y
            }) {
                chest.items.push(ChestItem {
                    slot: saved_chest_item.slot,
                    item_id: saved_chest_item.id,
                    amount: saved_chest_item.amount,
                });
            }
        }

        for saved_chest_spawn in save_data.chest_spawns {
            if let Some(chest) = self.chests.iter_mut().find(|chest| {
                chest.coords.x == saved_chest_spawn.x && chest.coords.y == saved_chest_spawn.y
            }) {
                if let Some(spawn) = chest
                    .spawns
                    .iter_mut()
                    .find(|spawn| spawn.slot == saved_chest_spawn.slot)
                {
                    match Utc.from_local_datetime(&saved_chest_spawn.taken) {
                        chrono::offset::LocalResult::Single(dt) => spawn.last_taken = dt,
                        _ => {
                            error!(
                                "Failed to convert saved chest spawn last_taken to DateTime<Utc>"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MapSaveData {
    rid: [i32; 2],
    items: Vec<SavedItem>,
    npcs: Vec<SavedNpc>,
    chest_items: Vec<SavedChestItem>,
    chest_spawns: Vec<SavedChestSpawn>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedItem {
    index: i32,
    id: i32,
    amount: i32,
    x: i32,
    y: i32,
    owner: i32,
    ticks: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedNpc {
    index: i32,
    x: i32,
    y: i32,
    direction: i32,
    hp: i32,
    alive: bool,
    ticks: i32,
    opponents: Vec<SavedNpcOpponent>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedNpcOpponent {
    id: i32,
    damage: i32,
    ticks: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedChestItem {
    x: i32,
    y: i32,
    slot: i32,
    id: i32,
    amount: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedChestSpawn {
    x: i32,
    y: i32,
    slot: i32,
    taken: NaiveDateTime,
}
