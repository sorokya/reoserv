use super::super::World;

impl World {
    pub fn set_character_property(&self, name: String, property: String, value: String) {
        let (player_id, player) = match self.characters.get(&name) {
            Some(player_id) => match self.players.get(player_id) {
                Some(player) => (*player_id, player.clone()),
                None => return,
            },
            None => return,
        };

        tokio::spawn(async move {
            let map = match player.get_map().await {
                Ok(map) => map,
                Err(e) => {
                    tracing::error!("Error getting map for set_character_property: {}", e);
                    return;
                }
            };

            map.set_character_property(player_id, property, value);
        });
    }
}
