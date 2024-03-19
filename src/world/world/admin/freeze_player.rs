use eolib::protocol::net::{server::WalkCloseServerPacket, PacketAction, PacketFamily};

use crate::LANG;

use super::super::World;

impl World {
    pub fn freeze_player(&mut self, victim_name: String, admin_name: String) {
        let player_id = match self.characters.get(&victim_name) {
            Some(player_id) => player_id,
            None => return,
        };

        let player = match self.players.get(player_id) {
            Some(player) => player,
            None => return,
        };

        player.send(
            PacketAction::Close,
            PacketFamily::Walk,
            &WalkCloseServerPacket::new(),
        );

        self.broadcast_server_message(&get_lang_string!(
            &LANG.announce_freeze,
            victim = victim_name,
            name = admin_name
        ));
    }
}
