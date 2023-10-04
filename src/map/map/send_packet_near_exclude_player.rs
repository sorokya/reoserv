use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily},
};

use crate::utils::in_range;

use super::Map;

impl Map {
    pub fn send_packet_near_exclude_player<T>(
        &self,
        coords: &Coords,
        exclude_player_id: EOShort,
        action: PacketAction,
        family: PacketFamily,
        packet: T,
    ) where
        T: Serializeable,
    {
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        let buf = builder.get();
        for (player_id, character) in self.characters.iter() {
            if *player_id != exclude_player_id && in_range(coords, &character.coords) {
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .send(action, family, buf.clone());
            }
        }
    }
}
