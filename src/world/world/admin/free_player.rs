use eolib::protocol::{net::server::WarpEffect, Coords};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use crate::SETTINGS;

use super::super::World;

impl World {
    pub fn free_player(&mut self, victim_name: String) {
        let mut player_online = false;
        if let Some(player_id) = self.characters.get(&victim_name) {
            if let Some(player) = self.players.get(player_id) {
                player_online = true;
                player.request_warp(
                    SETTINGS.jail.free_map,
                    Coords {
                        x: SETTINGS.jail.free_x,
                        y: SETTINGS.jail.free_y,
                    },
                    false,
                    Some(WarpEffect::Admin),
                );
            }
        }

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
                            "map" => SETTINGS.jail.free_map,
                            "x" => SETTINGS.jail.free_x,
                            "y" => SETTINGS.jail.free_y,
                            "name" => &victim_name,
                        },
                    )
                    .await
                {
                    error!("Failed to free player: {}", e);
                }
            });
        }
    }
}
