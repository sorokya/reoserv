use eolib::protocol::{net::server::WarpEffect, Coords};

use crate::{db::insert_params, LANG, SETTINGS};

use super::super::World;

impl World {
    pub fn jail_player(&mut self, victim_name: String, admin_name: String) {
        let mut player_online = false;
        if let Some(player_id) = self.characters.get(&victim_name) {
            if let Some(player) = self.players.get(player_id) {
                player_online = true;
                player.request_warp(
                    SETTINGS.jail.map,
                    Coords {
                        x: SETTINGS.jail.x,
                        y: SETTINGS.jail.y,
                    },
                    false,
                    Some(WarpEffect::Admin),
                );
            }
        }

        self.broadcast_server_message(&get_lang_string!(
            &LANG.announce_remove,
            victim = victim_name,
            name = admin_name,
            method = "jailed"
        ));

        if !player_online {
            let db = self.db.clone();
            tokio::spawn(async move {
                if let Err(e) = db
                    .execute(&insert_params(
                        include_str!("../../../sql/offline_jail.sql"),
                        &[
                            ("map", &SETTINGS.jail.map),
                            ("x", &SETTINGS.jail.x),
                            ("y", &SETTINGS.jail.y),
                            ("name", &victim_name),
                        ],
                    ))
                    .await
                {
                    error!("Failed to jail player: {}", e);
                }
            });
        }
    }
}
