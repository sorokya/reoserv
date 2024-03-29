use tokio::sync::oneshot;

use super::World;

impl World {
    pub fn drop_player(
        &mut self,
        player_id: i32,
        ip: String,
        account_id: i32,
        character_name: &str,
        guild_tag: Option<String>,
        respond_to: oneshot::Sender<()>,
    ) {
        self.connection_log.remove_connection(&ip);

        if !self.players.contains_key(&player_id) {
            let _ = respond_to.send(());
            return;
        }

        self.players.remove(&player_id);

        if account_id > 0 {
            self.accounts.retain(|id| *id != account_id);
        }

        if self.characters.contains_key(character_name) {
            self.characters.remove(character_name);
        }

        if let Some(guild_tag) = guild_tag {
            let remaining = match self.guilds.get_mut(&guild_tag) {
                Some(guild) => {
                    guild.retain(|id| *id != player_id);
                    guild.len()
                }
                None => 0,
            };

            if remaining == 0 {
                self.guilds.remove(&guild_tag);
            }
        }

        let _ = respond_to.send(());
    }
}
