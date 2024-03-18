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

        let guild_tag = match character.guild_tag {
            Some(ref guild_tag) => guild_tag.to_owned(),
            None => return,
        };

        character.guild_tag = None;
        character.guild_name = None;
        character.guild_rank = None;
        character.guild_rank_string = None;

        self.world.remove_guild_member(player_id, guild_tag);

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

        let mut character = character.to_owned();
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            if let Err(e) = character.save(&mut conn).await {
                error!("Error saving character: {}", e);
            }
        });
    }
}
