use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::GuildRequestServerPacket, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn send_guild_create_requests(&self, leader_player_id: i32, guild_identity: String) {
        let packet = GuildRequestServerPacket {
            player_id: leader_player_id,
            guild_identity,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildRequestServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for player in self.characters.iter().filter_map(|(player_id, character)| {
            if *player_id != leader_player_id && character.guild_tag.is_none() {
                character.player.as_ref()
            } else {
                None
            }
        }) {
            player.send_buf(PacketAction::Request, PacketFamily::Guild, buf.clone());
        }
    }
}
