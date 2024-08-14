use eolib::protocol::net::{server::GuildAgreeServerPacket, PacketAction, PacketFamily};

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
        character.guild_rank = Some(9);

        self.world.add_guild_member(player_id, guild_tag.clone());

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Agree,
                PacketFamily::Guild,
                &GuildAgreeServerPacket {
                    recruiter_id,
                    guild_tag,
                    guild_name,
                    rank_name: guild_rank_string,
                },
            );
        }
    }
}
