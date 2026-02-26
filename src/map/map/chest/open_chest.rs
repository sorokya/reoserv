use eolib::protocol::{
    Coords,
    net::{PacketAction, PacketFamily, ThreeItem, server::ChestOpenServerPacket},
    r#pub::ItemType,
};

use crate::{ITEM_DB, utils::in_client_range};

use super::super::Map;

impl Map {
    pub fn open_chest(&self, player_id: i32, coords: Coords) {
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

        if let Some(key) = chest.key
            && !character.items.iter().any(|item| {
                let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
                    Some(item_data) => item_data,
                    None => return false,
                };

                item_data.r#type == ItemType::Key && item_data.spec1 == key
            })
        {
            return;
        }

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        player.set_chest_index(chest_index);

        player.send(
            PacketAction::Open,
            PacketFamily::Chest,
            &ChestOpenServerPacket {
                coords,
                items: chest
                    .items
                    .iter()
                    .map(|item| ThreeItem {
                        id: item.item_id,
                        amount: item.amount,
                    })
                    .collect(),
            },
        );
    }
}
