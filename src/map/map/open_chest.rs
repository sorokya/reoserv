use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::chest, Coords, PacketAction, PacketFamily, ShortItem},
};

use crate::utils::in_client_range;

use super::Map;

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
