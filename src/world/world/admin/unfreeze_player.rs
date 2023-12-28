use bytes::Bytes;
use eolib::protocol::net::{PacketAction, PacketFamily};

use crate::LANG;

use super::super::World;

impl World {
    pub fn unfreeze_player(&mut self, victim_name: String, admin_name: String) {
        let player_id = match self.characters.get(&victim_name) {
            Some(player_id) => player_id,
            None => return,
        };

        let player = match self.players.get(player_id) {
            Some(player) => player,
            None => return,
        };

        player.send(
            PacketAction::Open,
            PacketFamily::Walk,
            Bytes::from_static(b"S"),
        );

        self.broadcast_server_message(&get_lang_string!(
            &LANG.announce_unfreeze,
            victim = victim_name,
            name = admin_name
        ));
    }
}
