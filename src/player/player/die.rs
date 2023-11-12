use eo::protocol::{Coords, WarpAnimation};

use super::Player;

impl Player {
    pub async fn die(&mut self) {
        let mut character = self
            .map
            .as_ref()
            .unwrap()
            .leave(self.id, Some(WarpAnimation::None), self.interact_player_id)
            .await;
        character.map_id = 0;
        character.coords = Coords { x: 0, y: 0 };
        self.character = Some(character);
        self.map = None;

        let character = self.character.as_ref().unwrap();
        self.request_warp(
            character.get_spawn_map(),
            character.get_spawn_coords(),
            false,
            Some(WarpAnimation::None),
        )
        .await;
    }
}
