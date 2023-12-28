use std::cmp;

use eolib::{protocol::{Coords, map::MapTileSpec, net::{PacketAction, PacketFamily}}, data::EoWriter};

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
            Some(tile) => *tile == MapTileSpec::BankVault,
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

        let mut writer = EoWriter::new();
        writer.add_short(item_id);
        writer.add_three(character.get_item_amount(item_id));

        let weight = character.get_weight();
        writer.add_char(weight.current);
        writer.add_char(weight.max);

        for item in &character.bank {
            writer.add_short(item.id);
            writer.add_three(item.amount);
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Get,
            PacketFamily::Locker,
            writer.to_byte_array(),
        );
    }
}
