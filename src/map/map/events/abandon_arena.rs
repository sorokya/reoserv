use bytes::Bytes;
use eolib::protocol::{
    net::{PacketAction, PacketFamily},
    Coords,
};

use super::super::Map;

impl Map {
    pub fn abandon_arena(&mut self) {
        let buf = Bytes::from_static(&b"The event was aborted, last opponent left -server"[..]);

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
