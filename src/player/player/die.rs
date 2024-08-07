use eolib::protocol::Coords;

use super::Player;

impl Player {
    pub async fn die(&mut self) {
        let mut character = self
            .map
            .as_ref()
            .unwrap()
            .leave(self.id, None, self.interact_player_id)
            .await;

        let spawn_map = character.get_spawn_map();
        let local = character.map_id == spawn_map;

        character.map_id = 0;
        character.coords = Coords { x: 0, y: 0 };

        let spawn_map = character.get_spawn_map();
        let spawn_coords = character.get_spawn_coords();

        self.character = Some(character.clone());

        let nirvana = self.world.get_map(0).await.unwrap();
        nirvana.enter(Box::new(character), None).await;
        self.map = Some(nirvana);

        self.request_warp(spawn_map, spawn_coords, local, None)
            .await;
    }
}
