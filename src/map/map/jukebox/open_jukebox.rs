use eolib::protocol::{
    map::MapTileSpec,
    net::{server::JukeboxOpenServerPacket, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn open_jukebox(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if self.player_in_range_of_tile(player_id, MapTileSpec::Jukebox) {
            character.player.as_ref().unwrap().send(
                PacketAction::Open,
                PacketFamily::Jukebox,
                &JukeboxOpenServerPacket {
                    map_id: self.id,
                    jukebox_player: if self.jukebox_ticks > 0 {
                        match self.jukebox_player {
                            Some(ref player) => player.clone(),
                            None => "Busy".to_string(), // just in case
                        }
                    } else {
                        String::new()
                    },
                },
            );
        }
    }
}
