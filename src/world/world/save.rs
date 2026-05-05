use super::World;

impl World {
    pub async fn save_async(&mut self) {
        if let Some(maps) = self.maps.as_ref() {
            const SAVE_BATCH_SIZE: usize = 4;

            let map_handles: Vec<_> = maps.values().collect();
            for batch in map_handles.chunks(SAVE_BATCH_SIZE) {
                let saves = batch.iter().map(|map| map.save_async());
                let results = futures::future::join_all(saves).await;
                for result in results {
                    if let Err(e) = result {
                        tracing::error!("Failed to save map: {}", e);
                    }
                }
            }
        }
    }
}
