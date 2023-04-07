use chrono::{Duration, Utc};
use eo::{protocol::{Coords, server::chest, ShortItem, PacketAction, PacketFamily}, pubs::EmfTileSpec, data::{StreamBuilder, Serializeable}};
use rand::seq::SliceRandom;

use crate::{map::{
    chest::{ChestItem, ChestSpawn},
    Chest,
}, utils::get_distance};

use super::Map;

impl Map {
    fn create_chests(&mut self) {
        let now = Utc::now();
        for item in &self.file.items {
            let item_coords = Coords {
                x: item.x,
                y: item.y,
            };

            if self
                .file
                .spec_rows
                .iter()
                .find(|row| row.y == item.y)
                .and_then(|row| {
                    row.tiles
                        .iter()
                        .find(|tile| tile.spec == EmfTileSpec::Chest && tile.x == item.x)
                })
                .is_none()
            {
                continue;
            }

            if let Some(chest) = self
                .chests
                .iter_mut()
                .find(|chest| chest.coords == item_coords)
            {
                if chest.key.is_none() && item.key_required != 0 {
                    chest.key = Some(item.key_required);
                }

                chest.spawns.push(ChestSpawn {
                    slot: item.chest_slot + 1,
                    item_id: item.item_id,
                    amount: item.item_amount,
                    spawn_time: item.spawn_time,
                    last_taken: now,
                });
            } else {
                self.chests.push(Chest {
                    coords: item_coords,
                    items: Vec::new(),
                    spawns: vec![ChestSpawn {
                        slot: item.chest_slot + 1,
                        item_id: item.item_id,
                        amount: item.item_amount,
                        spawn_time: item.spawn_time,
                        last_taken: now,
                    }],
                    key: match item.key_required {
                        0 => None,
                        key => Some(key),
                    },
                });
            }
        }

        // For any chests that don't have spawns
        for row in &self.file.spec_rows {
            for tile in &row.tiles {
                if tile.spec == EmfTileSpec::Chest
                    && !self
                        .chests
                        .iter()
                        .any(|chest| chest.coords.y == row.y && chest.coords.x == tile.x)
                {
                    self.chests.push(Chest {
                        coords: Coords {
                            x: tile.x,
                            y: row.y,
                        },
                        items: Vec::new(),
                        spawns: Vec::new(),
                        key: None,
                    });
                }
            }
        }
    }

    pub async fn spawn_items(&mut self) {
        if !self.file.items.is_empty() {
            if self.chests.is_empty() {
                self.create_chests();
            }

            let now = Utc::now();
            for chest in self.chests.iter_mut() {
                let max_slot = chest
                    .spawns
                    .iter()
                    .map(|spawn| spawn.slot)
                    .max()
                    .unwrap_or(0);

                let mut spawned_item = false;
                for slot in 1..=max_slot {
                    if chest.items.iter().any(|item| item.slot == slot) {
                        continue;
                    }

                    let possible_spawns = chest
                        .spawns
                        .iter()
                        .filter(|spawn| {
                            spawn.slot == slot
                                && now - spawn.last_taken
                                    >= Duration::minutes(spawn.spawn_time.into())
                        })
                        .collect::<Vec<_>>();
                    if possible_spawns.is_empty() {
                        continue;
                    }

                    let spawn = match possible_spawns.choose(&mut rand::thread_rng()) {
                        Some(spawn) => spawn,
                        None => {
                            error!("Failed to choose spawn");
                            continue;
                        }
                    };

                    chest.items.push(ChestItem {
                        slot,
                        item_id: spawn.item_id,
                        amount: spawn.amount,
                    });
                    spawned_item = true;
                }

                if spawned_item {
                    let packet = chest::Agree {
                        items: chest.items.iter().map(|item| ShortItem {
                            id: item.item_id,
                            amount: item.amount,
                        }).collect(),
                    };

                    let mut builder = StreamBuilder::new();
                    packet.serialize(&mut builder);
                    let buf = builder.get();

                    for character in self.characters.values() {
                        let distance = get_distance(&character.coords, &chest.coords);
                        if distance <= 1 {
                            character.player.as_ref().unwrap().send(PacketAction::Agree, PacketFamily::Chest, buf.clone());
                        }
                    }
                }
            }
        }
    }
}
