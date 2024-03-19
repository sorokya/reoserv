use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::TalkAdminServerPacket, PacketAction, PacketFamily},
        AdminLevel,
    },
};

use super::super::World;

impl World {
    pub async fn broadcast_admin_message(&self, name: &str, message: &str) {
        let packet = TalkAdminServerPacket {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkAdminServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != name
                    && i32::from(character.admin_level) >= i32::from(AdminLevel::Guardian)
                {
                    player.send_buf(PacketAction::Admin, PacketFamily::Talk, buf.clone());
                }
            }
        }
    }
}
