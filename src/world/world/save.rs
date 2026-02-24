use super::World;

impl World {
    pub fn save(&mut self) {
        if let Some(maps) = self.maps.as_ref() {
            for map in maps.values() {
                map.save();
            }
        }
    }

    pub async fn save_async(&mut self) {
        if let Some(maps) = self.maps.as_ref() {
            let saves = maps.values().map(|map| map.save_async());
            let results = futures::future::join_all(saves).await;
            for result in results {
                if let Err(e) = result {
                    error!("Failed to save map: {}", e);
                }
            }
        }
    }
}
