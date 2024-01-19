use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::GuildKickServerPacket, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn kick_from_guild(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.guild_tag = None;
        character.guild_name = None;
        character.guild_rank_index = None;
        character.guild_rank_string = None;

        let packet = GuildKickServerPacket::default();

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildKickServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Kick,
            PacketFamily::Guild,
            writer.to_byte_array(),
        );
    }
}
