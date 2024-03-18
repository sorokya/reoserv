use mysql_async::prelude::Queryable;
use mysql_common::params;

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

        let pool = self.pool.clone();

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

            // Wait a few seconds to ensure all characters have been saved.
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            if let Err(e) = conn
                .exec_drop(
                    include_str!("../../sql/delete_guild.sql"),
                    params! {
                        "tag" => &guild_tag,
                    },
                )
                .await
            {
                error!("Error deleting guild: {}", e);
            }
        });
    }
}
