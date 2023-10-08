use super::Player;

impl Player {
    pub async fn close(&mut self, reason: String) {
        self.queue.borrow_mut().clear();
        if let Some(map) = self.map.as_ref() {
            let mut character = map.leave(self.id, None).await;
            let pool = self.pool.clone();
            let _ = tokio::task::Builder::new()
                .name("character_save")
                .spawn(async move {
                    let mut conn = pool.get_conn().await.unwrap();
                    if let Some(logged_in_at) = character.logged_in_at {
                        let now = chrono::Utc::now();
                        character.usage += (now.timestamp() - logged_in_at.timestamp()) as u32 / 60;
                    }
                    character.save(&mut conn).await.unwrap();
                });
        }

        let character_name = self
            .character
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default();
        self.world
            .drop_player(self.id, self.account_id, character_name)
            .await
            .unwrap();
        info!("player {} connection closed: {:?}", self.id, reason);
    }
}
