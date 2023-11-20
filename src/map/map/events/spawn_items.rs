use chrono::{Duration, Utc};
use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{server::chest, PacketAction, PacketFamily, ShortItem},
};
use rand::seq::SliceRandom;

use crate::{map::chest::ChestItem, utils::get_distance};

use super::super::Map;

impl Map {
    pub async fn spawn_items(&mut self) {
        if !self.file.items.is_empty() {
            let now = Utc::now();
            let mut chest_index: usize = 0;
            for chest in self.chests.iter_mut() {
                chest_index += 1;
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
                        items: chest
                            .items
                            .iter()
                            .map(|item| ShortItem {
                                id: item.item_id,
                                amount: item.amount,
                            })
                            .collect(),
                    };

                    let mut builder = StreamBuilder::new();
                    packet.serialize(&mut builder);
                    let buf = builder.get();

                    for character in self.characters.values() {
                        let distance = get_distance(&character.coords, &chest.coords);
                        if distance > 1 {
                            continue;
                        }

                        let player = match character.player.as_ref() {
                            Some(player) => player,
                            None => continue,
                        };

                        let player_chest_index = match player.get_chest_index().await {
                            Some(index) => index,
                            None => continue,
                        };

                        if player_chest_index != chest_index - 1 {
                            continue;
                        }

                        character.player.as_ref().unwrap().send(
                            PacketAction::Agree,
                            PacketFamily::Chest,
                            buf.clone(),
                        );
                    }
                }
            }
        }
    }
}
