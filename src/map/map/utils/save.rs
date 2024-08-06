use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub async fn save(&mut self, respond_to: oneshot::Sender<()>) {
        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get connection from pool: {}", e);
                let _ = respond_to.send(());
                return;
            }
        };

        let now = chrono::Utc::now();

        for character in self.characters.values_mut() {
            if let Some(logged_in_at) = character.logged_in_at {
                character.usage += (now.timestamp() - logged_in_at.timestamp()) as i32 / 60;
            }

            if let Err(e) = character.save(&mut conn).await {
                error!("Failed to update character: {}", e);
                continue;
            }
        }

        let _ = respond_to.send(());
    }
}
