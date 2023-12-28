use std::cmp;

use eo::{
    data::{i32, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily},
    pubs::EmfTileSpec,
};

use super::super::Map;

impl Map {
    pub fn take_locker_item(&mut self, player_id: i32, item_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let adjacent_tiles = [
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y - 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y + 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x - 1,
                y: character.coords.y,
            }),
            self.get_tile(&Coords {
                x: character.coords.x + 1,
                y: character.coords.y,
            }),
        ];

        if !adjacent_tiles.iter().any(|tile| match tile {
            Some(tile) => *tile == EmfTileSpec::BankVault,
            None => false,
        }) {
            return;
        }

        let amount = character.get_bank_item_amount(item_id);
        if amount == 0 {
            return;
        }

        let amount = cmp::min(character.can_hold(item_id, amount), amount);
        if amount == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_bank_item(item_id, amount);
        character.add_item(item_id, amount);

        let mut builder = StreamBuilder::new();
        builder.add_short(item_id);
        builder.add_three(character.get_item_amount(item_id));

        let weight = character.get_weight();
        builder.add_char(weight.current);
        builder.add_char(weight.max);

        for item in &character.bank {
            builder.add_short(item.id);
            builder.add_three(item.amount);
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Get,
            PacketFamily::Locker,
            builder.get(),
        );
    }
}
