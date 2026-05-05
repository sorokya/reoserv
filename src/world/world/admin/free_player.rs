use eolib::protocol::{Coords, net::server::WarpEffect};

use crate::{SETTINGS, db::insert_params};

use super::super::World;

impl World {
    pub fn free_player(&mut self, victim_name: String) {
        let mut player_online = false;
        if let Some(player_id) = self.characters.get(&victim_name)
            && let Some(player) = self.players.get(player_id)
        {
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

        if !player_online {
            let db = self.db.clone();
            tokio::spawn(async move {
                if let Err(e) = db
                    .execute(&insert_params(
                        include_str!("../../../sql/offline_jail.sql"),
                        &[
                            ("map", &SETTINGS.jail.free_map),
                            ("x", &SETTINGS.jail.free_x),
                            ("y", &SETTINGS.jail.free_y),
                            ("name", &victim_name),
                        ],
                    ))
                    .await
                {
                    tracing::error!("Failed to free player: {}", e);
                }
            });
        }
    }
}
