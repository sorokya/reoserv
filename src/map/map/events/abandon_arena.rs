use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::TalkServerServerPacket, PacketAction, PacketFamily},
        Coords,
    },
};

use super::super::Map;

impl Map {
    pub fn abandon_arena(&mut self) {
        let packet = TalkServerServerPacket {
            message: "The event was aborted, last opponent left -server".to_string(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkServerServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for player in &self.arena_players {
            let character = match self.characters.get(&player.player_id) {
                Some(character) => character,
                None => continue,
            };

            character.player.as_ref().unwrap().arena_die(Coords {
                x: self.file.relog_x,
                y: self.file.relog_y,
            });

            character.player.as_ref().unwrap().send(
                PacketAction::Server,
                PacketFamily::Talk,
                buf.clone(),
            );
        }

        self.arena_players.clear();
    }
}
