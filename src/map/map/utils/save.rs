use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub async fn save(&mut self, respond_to: oneshot::Sender<()>) {
        let mut conn = self.pool.get_conn().await.unwrap();
        let now = chrono::Utc::now();

        for character in self.characters.values_mut() {
            if let Some(logged_in_at) = character.logged_in_at {
                character.usage += (now.timestamp() - logged_in_at.timestamp()) as u32 / 60;
            }
            character.save(&mut conn).await.unwrap();
        }

        // TODO: Save map state

        let _ = respond_to.send(());
    }
}
