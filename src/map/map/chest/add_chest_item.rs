use std::cmp;

use bytes::Bytes;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{ChestAgreeServerPacket, ChestReplyServerPacket},
        Item, PacketAction, PacketFamily, ThreeItem,
    },
};

use crate::{
    map::{chest::ChestItem, Chest},
    utils::get_distance,
    SETTINGS,
};

use super::super::Map;

impl Map {
    pub async fn add_chest_item(&mut self, player_id: i32, item: Item) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let chest_index = match player.get_chest_index().await {
            Some(index) => index,
            None => return,
        };

        if player.is_trading().await {
            return;
        }

        let chest: &Chest = match self.chests.get(chest_index) {
            Some(chest) => chest,
            None => return,
        };

        if get_distance(&character.coords, &chest.coords) > 1 {
            return;
        }

        let amount = cmp::min(item.amount, character.get_item_amount(item.id));
        if amount == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let chest: &mut Chest = match self.chests.get_mut(chest_index) {
            Some(chest) => chest,
            None => return,
        };

        let user_items = chest.items.iter().filter(|i| i.slot == 0).count();
        let mut chest_slots: Vec<i32> = vec![];

        for spawn in chest.spawns.iter() {
            if !chest_slots.contains(&spawn.slot) {
                chest_slots.push(spawn.slot);
            }
        }

        let mut chest_full = false;
        if let Some(existing_item) = chest.items.iter_mut().find(|i| i.item_id == item.id) {
            if existing_item.amount + amount > SETTINGS.limits.max_chest {
                chest_full = true;
            } else {
                character.remove_item(item.id, amount);
                existing_item.amount += amount;
            }
        } else if chest_slots.len() + user_items < SETTINGS.chest.slots as usize {
            character.remove_item(item.id, amount);
            chest.items.push(ChestItem {
                slot: 0,
                item_id: item.id,
                amount,
            });
        } else {
            chest_full = true;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        if chest_full {
            player.send_buf(
                PacketAction::Spec,
                PacketFamily::Chest,
                Bytes::from(vec![0]),
            );
            return;
        }

        let items: Vec<ThreeItem> = chest
            .items
            .iter()
            .map(|i| ThreeItem {
                id: i.item_id,
                amount: i.amount,
            })
            .collect();

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Chest,
                &ChestReplyServerPacket {
                    added_item_id: item.id,
                    remaining_amount: character.get_item_amount(item.id),
                    weight: character.get_weight(),
                    items: items.clone(),
                },
            );
        }

        let packet = ChestAgreeServerPacket { items };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize ChestAgreeServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for (id, character) in self.characters.iter() {
            let distance = get_distance(&character.coords, &chest.coords);
            if *id == player_id || distance > 1 {
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

            if player_chest_index != chest_index {
                continue;
            }

            player.send_buf(PacketAction::Agree, PacketFamily::Chest, buf.clone());
        }
    }
}
