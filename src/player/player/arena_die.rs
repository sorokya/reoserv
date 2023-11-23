use eo::protocol::{Coords, WarpAnimation};

use super::Player;

impl Player {
    pub async fn arena_die(&mut self, spawn_coords: Coords) {
        let mut character = self
            .map
            .as_ref()
            .unwrap()
            .leave(self.id, Some(WarpAnimation::None), self.interact_player_id)
            .await;

        let current_map = character.map_id;
        character.map_id = 0;
        character.coords = Coords { x: 0, y: 0 };
        self.character = Some(character);
        self.map = None;

        self.request_warp(current_map, spawn_coords, false, Some(WarpAnimation::None))
            .await;
    }
}
