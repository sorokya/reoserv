use super::World;

impl World {
    pub async fn save(&mut self) {
        if let Some(maps) = self.maps.as_ref() {
            for map in maps.values() {
                map.save().await;
            }
        }
    }
}
