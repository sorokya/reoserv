use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::chest, Coords, PacketAction, PacketFamily, ShortItem},
    pubs::EifItemType,
};

use crate::{utils::in_client_range, ITEM_DB};

use super::super::Map;

impl Map {
    pub fn open_chest(&self, player_id: EOShort, coords: Coords) {
        let chest = match self.chests.iter().find(|chest| chest.coords == coords) {
            Some(chest) => chest,
            None => return,
        };

        let chest_index = self
            .chests
            .iter()
            .position(|chest| chest.coords == coords)
            .unwrap();

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !in_client_range(&character.coords, &coords) {
            return;
        }

        if let Some(key) = chest.key {
            if !character.items.iter().any(|item| {
                let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
                    Some(item_data) => item_data,
                    None => return false,
                };

                item_data.r#type == EifItemType::Key && item_data.spec1 as EOShort == key
            }) {
                return;
            }
        }

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        player.set_chest_index(chest_index);

        let reply = chest::Open {
            coords,
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
        reply.serialize(&mut builder);
        character.player.as_ref().unwrap().send(
            PacketAction::Open,
            PacketFamily::Chest,
            builder.get(),
        );
    }
}
