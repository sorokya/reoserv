use chrono::Utc;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{ChestAgreeServerPacket, ChestGetServerPacket},
        PacketAction, PacketFamily, ThreeItem,
    },
};

use crate::{map::Chest, utils::get_distance};

use super::super::Map;

impl Map {
    // TODO: Make sure player can hold item before taking
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

        let remaining_items: Vec<ThreeItem> = chest
            .items
            .iter()
            .map(|item| ThreeItem {
                id: item.item_id,
                amount: item.amount,
            })
            .collect();

        character.add_item(item.item_id, item.amount);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Get,
                PacketFamily::Chest,
                &ChestGetServerPacket {
                    taken_item: ThreeItem {
                        id: item.item_id,
                        amount: item.amount,
                    },
                    weight: character.get_weight(),
                    items: remaining_items.clone(),
                },
            );
        }

        let packet = ChestAgreeServerPacket {
            items: remaining_items,
        };

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
