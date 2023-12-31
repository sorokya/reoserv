use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{PacketAction, PacketFamily},
        Coords,
    },
};

use crate::utils::in_range;

use super::super::Map;

impl Map {
    pub fn send_packet_near_exclude_player<T>(
        &self,
        coords: &Coords,
        exclude_player_id: i32,
        action: PacketAction,
        family: PacketFamily,
        packet: T,
    ) where
        T: EoSerialize,
    {
        let mut writer = EoWriter::new();
        packet.serialize(&mut writer);
        let buf = writer.to_byte_array();
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
