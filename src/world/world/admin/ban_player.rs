use super::super::World;
use crate::{LANG, db::insert_params};

impl World {
    pub async fn ban_player(
        &mut self,
        victim_name: String,
        admin_name: String,
        duration: String,
        silent: bool,
    ) {
        if let Some(player_id) = self.characters.get(&victim_name)
            && let Some(player) = self.players.get(player_id)
        {
            player.close("Player banned".to_string());
        }

        if !silent {
            self.broadcast_server_message(&get_lang_string!(
                &LANG.announce_remove,
                victim = victim_name,
                name = admin_name,
                method = "banned"
            ));
        }

        let db = self.db.clone();
        tokio::spawn(async move {
            let row = match db
                .query_one(&insert_params(
                    include_str!("../../../sql/get_character_account.sql"),
                    &[("character_name", &victim_name)],
                ))
                .await
            {
                Ok(Some(row)) => row,
                Err(err) => {
                    tracing::error!("Failed to get account from database: {}", err);
                    return;
                }
                _ => return,
            };

            let account_id = match row.get_int(0) {
                Some(account_id) => account_id,
                None => return,
            };

            let ip = match row.get_string(1) {
                Some(ip) => ip,
                None => return,
            };

            let duration = duration_str::parse(&duration);

            match db
                .execute(&insert_params(
                    include_str!("../../../sql/create_ban.sql"),
                    &[
                        ("account_id", &account_id),
                        ("ip", &ip),
                        (
                            "duration",
                            &match duration {
                                Ok(duration) => Some(format!("{}", duration.as_secs() / 60)),
                                Err(_) => None,
                            },
                        ),
                    ],
                ))
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    tracing::error!("Failed to ban player: {}", err);
                }
            }
        });
    }
}
