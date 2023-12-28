use chrono::Utc;
use eo::{
    data::{i32, Serializeable, StreamBuilder},
    protocol::{server::chest, PacketAction, PacketFamily, ShortItem},
};

use crate::{map::Chest, utils::get_distance};

use super::super::Map;

impl Map {
    pub async fn take_chest_item(&mut self, player_id: i32, item_id: i32) {
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

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let chest: &mut Chest = match self.chests.get_mut(chest_index) {
            Some(chest) => chest,
            None => return,
        };

        let item_index = match chest.items.iter().position(|item| item.item_id == item_id) {
            Some(index) => index,
            None => return,
        };

        let item = chest.items.remove(item_index);
        if let Some(spawn) = chest
            .spawns
            .iter_mut()
            .find(|spawn| spawn.slot == item.slot)
        {
            spawn.last_taken = Utc::now();
        }

        let remaining_items: Vec<ShortItem> = chest
            .items
            .iter()
            .map(|item| ShortItem {
                id: item.item_id,
                amount: item.amount,
            })
            .collect();

        character.add_item(item.item_id, item.amount);

        let reply = chest::Get {
            taken_item: ShortItem {
                id: item.item_id,
                amount: item.amount,
            },
            weight: character.get_weight(),
            items: remaining_items.clone(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        character.player.as_ref().unwrap().send(
            PacketAction::Get,
            PacketFamily::Chest,
            builder.get(),
        );

        let packet = chest::Agree {
            items: remaining_items,
        };

        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
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
