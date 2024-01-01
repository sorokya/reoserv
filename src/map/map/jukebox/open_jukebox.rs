use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTileSpec,
        net::{server::JukeboxOpenServerPacket, PacketAction, PacketFamily},
    },
};

use super::super::Map;

impl Map {
    pub fn open_jukebox(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if self.player_in_range_of_tile(player_id, MapTileSpec::Jukebox) {
            let packet = JukeboxOpenServerPacket {
                map_id: self.id,
                jukebox_player: match self.jukebox_player {
                    Some(ref player) => player.clone(),
                    None => String::new(),
                },
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize JukeboxOpenServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::Open,
                PacketFamily::Jukebox,
                writer.to_byte_array(),
            );
        }
    }
}
