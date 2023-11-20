use super::super::Map;

impl Map {
    pub fn timed_warp_suck(&mut self) {
        for character in self.characters.values() {
            let coords = self.get_adjacent_tiles(&character.coords);
            let warp = match coords
                .iter()
                .map(|coords| self.get_warp(coords))
                .find(|warp| match warp {
                    Some(warp) => warp.door <= 1 && warp.level_required <= character.level,
                    None => false,
                }) {
                Some(warp) => warp.unwrap(),
                None => continue,
            };

            character.player.as_ref().unwrap().request_warp(
                warp.map,
                warp.coords,
                warp.map == self.id,
                None,
            );
        }
    }
}
