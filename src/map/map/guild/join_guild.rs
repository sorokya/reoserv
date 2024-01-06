use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::GuildAgreeServerPacket, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn join_guild(
        &mut self,
        player_id: i32,
        recruiter_id: i32,
        guild_tag: String,
        guild_name: String,
        guild_rank_string: String,
    ) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.guild_tag = Some(guild_tag.clone());
        character.guild_name = Some(guild_name.clone());
        character.guild_rank_string = Some(guild_rank_string.clone());
        character.guild_rank_index = Some(8);

        let packet = GuildAgreeServerPacket {
            recruiter_id,
            guild_tag,
            guild_name,
            rank_name: guild_rank_string,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildAgreeServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Agree,
            PacketFamily::Guild,
            writer.to_byte_array(),
        );
    }
}
