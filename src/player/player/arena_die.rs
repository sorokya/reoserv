use eolib::protocol::Coords;

use super::Player;

impl Player {
    pub async fn arena_die(&mut self, spawn_coords: Coords) {
        let mut character = match self
            .map
            .as_ref()
            .unwrap()
            .leave(self.id, None, self.interact_player_id)
            .await
        {
            Ok(character) => character,
            Err(e) => {
                self.close(format!("Failed to leave map: {}", e)).await;
                return;
            }
        };

        let current_map = character.map_id;

        character.map_id = 0;
        character.coords = Coords { x: 0, y: 0 };

        self.character = Some(character.clone());

        let nirvana = self.world.get_map(0).await.unwrap();
        nirvana
            .enter(Box::new(character), None)
            .await
            .expect("Failed to enter nirvana map. Timeout");
        self.map = Some(nirvana);

        self.request_warp(current_map, spawn_coords, false, None)
            .await;
    }
}
