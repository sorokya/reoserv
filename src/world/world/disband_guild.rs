use crate::db::insert_params;

use super::World;

impl World {
    pub fn disband_guild(&mut self, guild_tag: String) {
        let online_guild_player_ids = match self.guilds.remove(&guild_tag) {
            Some(online_guild_players) => online_guild_players,
            None => return,
        };

        let online_players = online_guild_player_ids
            .iter()
            .filter_map(|id| self.players.get(id))
            .map(|player| player.to_owned())
            .collect::<Vec<_>>();

        let db = self.db.clone();

        tokio::spawn(async move {
            for (index, player) in online_players.iter().enumerate() {
                let player_id = online_guild_player_ids[index];
                let map = match player.get_map().await {
                    Ok(map) => map,
                    Err(e) => {
                        error!("Error getting map: {}", e);
                        continue;
                    }
                };

                map.kick_from_guild(player_id);
            }

            if let Err(e) = db
                .execute(&insert_params(
                    include_str!("../../sql/delete_guild.sql"),
                    &[("tag", &guild_tag)],
                ))
                .await
            {
                error!("Error deleting guild: {}", e);
            }

            if let Err(e) = db
                .execute(include_str!("../../sql/cleanup_guildless_characters.sql"))
                .await
            {
                error!("Error cleaning up guildless characters: {}", e);
            }
        });
    }
}
