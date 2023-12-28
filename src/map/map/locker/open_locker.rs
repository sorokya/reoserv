use eo::{
    data::{i32, Serializeable, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily, ShortItem},
    pubs::EmfTileSpec,
};

use super::super::Map;

impl Map {
    pub fn open_locker(&self, player_id: i32) {
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

        if adjacent_tiles.iter().any(|tile| match tile {
            Some(tile) => *tile == EmfTileSpec::BankVault,
            None => false,
        }) {
            let packet = eo::protocol::server::locker::Open {
                locker_coords: character.coords,
                locker_items: character
                    .bank
                    .iter()
                    .map(|item| ShortItem {
                        id: item.id,
                        amount: item.amount,
                    })
                    .collect(),
            };

            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);

            character.player.as_ref().unwrap().send(
                PacketAction::Open,
                PacketFamily::Locker,
                builder.get(),
            );
        }
    }
}
