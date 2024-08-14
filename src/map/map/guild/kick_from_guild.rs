use eolib::protocol::net::{server::GuildKickServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn kick_from_guild(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let guild_tag = match character.guild_tag {
            Some(ref guild_tag) => guild_tag.to_owned(),
            None => return,
        };

        character.guild_tag = None;
        character.guild_name = None;
        character.guild_rank = None;
        character.guild_rank_string = None;

        self.world.remove_guild_member(player_id, guild_tag);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Kick,
                PacketFamily::Guild,
                &GuildKickServerPacket::default(),
            );
        }
    }
}
