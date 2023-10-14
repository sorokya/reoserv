use std::cmp;

use bytes::Bytes;
use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{Item, PacketAction, PacketFamily},
};

use crate::{
    map::{chest::ChestItem, Chest},
    utils::get_distance,
    SETTINGS,
};

use super::Map;

impl Map {
    pub async fn add_chest_item(&mut self, player_id: EOShort, item: Item) {
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

        let mut chest_full = false;
        if let Some(existing_item) = chest.items.iter_mut().find(|i| i.item_id == item.id) {
            if existing_item.amount + amount > SETTINGS.limits.max_chest {
                chest_full = true;
            } else {
                character.remove_item(item.id, amount);
                existing_item.amount += amount;
            }
        } else if chest.spawns.len() + user_items < SETTINGS.chest.slots as usize {
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
            player.send(
                PacketAction::Spec,
                PacketFamily::Chest,
                Bytes::from(vec![0]),
            );
            return;
        }

        let mut builder = StreamBuilder::new();
        builder.add_short(item.id);
        builder.add_int(character.get_item_amount(item.id));
        let weight = character.get_weight();
        builder.add_char(weight.current);
        builder.add_char(weight.max);

        for item in chest.items.iter() {
            builder.add_short(item.item_id);
            builder.add_three(item.amount);
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Chest,
            builder.get(),
        );

        let mut builder = StreamBuilder::new();
        for item in chest.items.iter() {
            builder.add_short(item.item_id);
            builder.add_three(item.amount);
        }

        let buf = builder.get();

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

            character.player.as_ref().unwrap().send(
                PacketAction::Agree,
                PacketFamily::Chest,
                buf.clone(),
            );
        }
    }
}
