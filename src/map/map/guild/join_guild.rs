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

        // TODO: This is potentially unsafe if character changes before being saved..
        // Possible duping hazard?
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
