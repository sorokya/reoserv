use eo::protocol::{Coords, WarpAnimation};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use crate::SETTINGS;

use super::super::World;

impl World {
    pub fn jail_player(&mut self, victim_name: String, admin_name: String) {
        let mut player_online = false;
        if let Some(player_id) = self.characters.get(&victim_name) {
            if let Some(player) = self.players.get(&player_id) {
                player_online = true;
                player.request_warp(
                    SETTINGS.jail.map,
                    Coords {
                        x: SETTINGS.jail.x,
                        y: SETTINGS.jail.y,
                    },
                    false,
                    Some(WarpAnimation::Admin),
                );
            }
        }

        self.broadcast_server_message(&format!(
            "Attention!! {} has been removed from the game -{} [jailed]",
            victim_name, admin_name
        ));

        let pool = self.pool.clone();
        if !player_online {
            tokio::spawn(async move {
                let mut conn = match pool.get_conn().await {
                    Ok(conn) => conn,
                    Err(err) => {
                        error!("Failed to get connection from pool: {}", err);
                        return;
                    }
                };

                if let Err(e) = conn
                    .exec_drop(
                        include_str!("../../../sql/offline_jail.sql"),
                        params! {
                            "map" => SETTINGS.jail.map,
                            "x" => SETTINGS.jail.x,
                            "y" => SETTINGS.jail.y,
                            "name" => &victim_name,
                        },
                    )
                    .await
                {
                    error!("Failed to jail player: {}", e);
                }
            });
        }
    }
}
