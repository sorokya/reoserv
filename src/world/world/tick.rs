use crate::{utils::ticks_since, SETTINGS};

use super::World;

impl World {
    pub async fn tick(&mut self) {
        let maps = match self.maps {
            Some(ref maps) => maps.values(),
            None => return,
        };

        for map in maps {
            map.act_npcs();

            if ticks_since(&self.last_npc_spawn_tick) >= SETTINGS.npcs.respawn_rate {
                map.spawn_npcs();
            }

            if ticks_since(&self.last_item_spawn_tick) >= SETTINGS.world.chest_spawn_rate {
                map.spawn_items();
            }

            if ticks_since(&self.last_player_recover_tick) >= SETTINGS.world.recover_rate {
                map.recover_players();
            }

            if ticks_since(&self.last_npc_recover_tick) >= SETTINGS.world.npc_recover_rate {
                map.recover_npcs();
            }
        }
    }
}
