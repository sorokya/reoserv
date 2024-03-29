use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkAnnounceServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn broadcast_announcement(&self, name: &str, message: &str) {
        let packet = TalkAnnounceServerPacket {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkAnnounceServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != name {
                    player.send_buf(PacketAction::Announce, PacketFamily::Talk, buf.clone());
                }
            }
        }
    }
}
