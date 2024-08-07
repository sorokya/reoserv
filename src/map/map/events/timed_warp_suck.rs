use std::cmp;

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_warp_suck(&mut self) {
        let player_ids = self
            .characters
            .keys()
            .copied()
            .collect::<Vec<_>>();

        for id in player_ids {
            let (coords, level, player) = {
                let character = match self.characters.get_mut(&id) {
                    Some(character) => character,
                    None => continue,
                };

                let player = match character.player.as_ref() {
                    Some(player) => player.to_owned(),
                    None => continue,
                };

                character.warp_suck_ticks = cmp::max(0, character.warp_suck_ticks - 1);

                if character.warp_suck_ticks > 0 {
                    continue;
                }

                character.warp_suck_ticks = SETTINGS.world.warp_suck_rate;

                (character.coords, character.level, player)
            };

            let coords = self.get_adjacent_tiles(&coords);
            let warp = match coords
                .iter()
                .map(|coords| self.get_warp(coords))
                .find(|warp| match warp {
                    Some(warp) => warp.door <= 1 && warp.level_required <= level,
                    None => false,
                }) {
                Some(warp) => warp.unwrap(),
                None => continue,
            };

            player.request_warp(
                warp.destination_map,
                warp.destination_coords,
                warp.destination_map == self.id,
                None,
            );
        }
    }
}
