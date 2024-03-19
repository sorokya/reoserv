use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkRequestServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn broadcast_guild_message(
        &self,
        player_id: Option<i32>,
        guild_tag: String,
        name: String,
        message: String,
    ) {
        if let Some(members) = self.guilds.get(&guild_tag) {
            let packet = TalkRequestServerPacket {
                player_name: name,
                message,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing TalkRequestServerPacket: {}", e);
                return;
            }

            let buf = writer.to_byte_array();

            members
                .iter()
                .filter(|member_id| player_id != Some(**member_id))
                .for_each(|member_id| {
                    if let Some(player) = self.players.get(member_id) {
                        player.send_buf(PacketAction::Request, PacketFamily::Talk, buf.clone());
                    }
                });
        }
    }
}
