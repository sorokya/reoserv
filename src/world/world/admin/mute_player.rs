use eolib::protocol::net::{server::TalkSpecServerPacket, PacketAction, PacketFamily};

use crate::LANG;

use super::super::World;

impl World {
    // TODO: Server side mute?
    // https://github.com/sorokya/reoserv/issues/187
    pub fn mute_player(&mut self, victim_name: String, admin_name: String) {
        let player_id = match self.characters.get(&victim_name) {
            Some(player_id) => player_id,
            None => return,
        };

        let player = match self.players.get(player_id) {
            Some(player) => player,
            None => return,
        };

        player.send(
            PacketAction::Spec,
            PacketFamily::Talk,
            &TalkSpecServerPacket {
                admin_name: admin_name.to_owned(),
            },
        );

        self.broadcast_server_message(&get_lang_string!(
            &LANG.announce_mute,
            victim = victim_name,
            name = admin_name
        ));
    }
}
