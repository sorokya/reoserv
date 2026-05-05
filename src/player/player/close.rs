use super::Player;

impl Player {
    pub async fn close(&mut self, reason: String) {
        self.queue.borrow_mut().clear();

        if let Some(map) = self.map.as_ref() {
            match map.leave(self.id, None, self.interact_player_id).await {
                Ok(character) => {
                    self.character = Some(character);
                }
                Err(e) => {
                    tracing::error!("Failed to leave map: {}", e);
                }
            }
        }

        if let Some(character) = self.character.as_mut()
            && let Err(e) = character.save(&self.db).await
        {
            tracing::error!("Failed to save character: {}", e);
        }

        self.world.remove_party_member(self.id, self.id);

        if let Err(e) = self
            .world
            .drop_player(
                self.id,
                self.ip.clone(),
                self.account_id,
                &self.character_name,
                &self.guild_tag,
            )
            .await
        {
            tracing::error!("Failed to drop player: {}", e);
        }

        self.closed = true;

        tracing::info!("player {} connection closed: {:?}", self.id, reason);
    }
}
