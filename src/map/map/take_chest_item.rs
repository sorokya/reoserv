use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::chest, Coords, PacketAction, PacketFamily, ShortItem, Weight},
};

use crate::utils::get_distance;

use super::Map;

impl Map {
    pub fn take_chest_item(&mut self, player_id: EOShort, coords: Coords, item_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if get_distance(&character.coords, &coords) > 1 {
            return;
        }

        let chest = match self.chests.iter_mut().find(|chest| chest.coords == coords) {
            Some(chest) => chest,
            None => return,
        };

        let item_index = match chest.items.iter().position(|item| item.item_id == item_id) {
            Some(index) => index,
            None => return,
        };

        let item = chest.items.remove(item_index);
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
            weight: Weight {
                current: character.weight,
                max: character.max_weight,
            },
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
            if *id != player_id && distance <= 1 {
                character.player.as_ref().unwrap().send(
                    PacketAction::Agree,
                    PacketFamily::Chest,
                    buf.clone(),
                );
            }
        }
    }
}
