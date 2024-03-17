use super::Player;

impl Player {
    pub async fn close(&mut self, reason: String) {
        self.queue.borrow_mut().clear();
        let (character_name, guild_tag) = if let Some(map) = self.map.as_ref() {
            let mut character = map.leave(self.id, None, self.interact_player_id).await;
            let character_name = character.name.clone();
            let guild_tag = character.guild_tag.clone();
            let pool = self.pool.clone();
            let _ = tokio::task::Builder::new()
                .name("character_save")
                .spawn(async move {
                    let mut conn = pool.get_conn().await.unwrap();
                    if let Some(logged_in_at) = character.logged_in_at {
                        let now = chrono::Utc::now();
                        character.usage += (now.timestamp() - logged_in_at.timestamp()) as i32 / 60;
                    }
                    character.save(&mut conn).await.unwrap();
                });
            (character_name, guild_tag)
        } else {
            self.character
                .as_ref()
                .map(|c| (c.name.clone(), c.guild_tag.clone()))
                .unwrap_or_default()
        };

        self.world.remove_party_member(self.id, self.id);

        self.world
            .drop_player(
                self.id,
                self.ip.clone(),
                self.account_id,
                character_name,
                guild_tag,
            )
            .await
            .unwrap();
        info!("player {} connection closed: {:?}", self.id, reason);
    }
}
