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

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !in_client_range(&character.coords, &coords) {
            return;
        }

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

        debug!("{:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        character.player.as_ref().unwrap().send(
            PacketAction::Open,
            PacketFamily::Chest,
            builder.get(),
        );
    }
}
