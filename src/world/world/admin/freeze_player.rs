use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::WalkCloseServerPacket, PacketAction, PacketFamily},
};

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

        let packet = WalkCloseServerPacket::new();

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing WalkCloseServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Close,
            PacketFamily::Walk,
            writer.to_byte_array(),
        );

        self.broadcast_server_message(&get_lang_string!(
            &LANG.announce_freeze,
            victim = victim_name,
            name = admin_name
        ));
    }
}
