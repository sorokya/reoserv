use mysql_async::{params, prelude::Queryable, Row};
use mysql_common::Value;

use super::super::World;
use crate::LANG;

impl World {
    pub async fn ban_player(
        &mut self,
        victim_name: String,
        admin_name: String,
        duration: String,
        silent: bool,
    ) {
        if let Some(player_id) = self.characters.get(&victim_name) {
            if let Some(player) = self.players.get(player_id) {
                player.close("Player banned".to_string());
            }
        }

        if !silent {
            self.broadcast_server_message(&get_lang_string!(
                &LANG.announce_remove,
                victim = victim_name,
                name = admin_name,
                method = "banned"
            ));
        }

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(err) => {
                    error!("Failed to get connection from pool: {}", err);
                    return;
                }
            };

            let row: Row = match conn
                .exec_first(
                    include_str!("../../../sql/get_character_account.sql"),
                    params! {
                        "character_name" => &victim_name,
                    },
                )
                .await
            {
                Ok(Some(row)) => row,
                Err(err) => {
                    error!("Failed to get account from database: {}", err);
                    return;
                }
                _ => return,
            };

            let account_id: u32 = match row.get("id") {
                Some(account_id) => account_id,
                None => return,
            };

            let ip: String = match row.get("last_login_ip") {
                Some(ip) => ip,
                None => return,
            };

            let duration = duration_str::parse(&duration);

            match conn
                .exec_drop(
                    include_str!("../../../sql/create_ban.sql"),
                    params! {
                        "account_id" => account_id,
                        "admin_name" => &admin_name,
                        "ip" => &ip,
                        "duration" => &match duration {
                            Ok(duration) => Value::from(format!("{}", duration.as_secs() / 60)),
                            Err(_) => Value::NULL,
                        },
                    },
                )
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to ban player: {}", err);
                }
            }
        });
    }
}
