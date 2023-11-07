use chrono::Utc;

use crate::{utils::ticks_since, SETTINGS};

use super::World;

impl World {
    pub async fn tick(&mut self) {
        let maps = match self.maps {
            Some(ref maps) => maps.values(),
            None => return,
        };

        let mut spawned_npcs = false;
        let mut spawned_items = false;
        let mut recovered_players = false;
        let mut recovered_npcs = false;
        let mut triggered_quakes = false;

        for map in maps {
            map.act_npcs();

            if ticks_since(&self.last_npc_spawn_tick) >= SETTINGS.npcs.respawn_rate {
                map.spawn_npcs();
                spawned_npcs = true;
            }

            if ticks_since(&self.last_item_spawn_tick) >= SETTINGS.world.chest_spawn_rate {
                map.spawn_items();
                spawned_items = true;
            }

            if ticks_since(&self.last_player_recover_tick) >= SETTINGS.world.recover_rate {
                map.recover_players();
                recovered_players = true;
            }

            if ticks_since(&self.last_npc_recover_tick) >= SETTINGS.world.npc_recover_rate {
                map.recover_npcs();
                recovered_npcs = true;
            }

            if ticks_since(&self.last_quake_tick) >= SETTINGS.map.quake_rate {
                map.timed_quake();
                triggered_quakes = true;
            }
        }

        if spawned_npcs {
            self.last_npc_spawn_tick = Utc::now();
        }

        if spawned_items {
            self.last_item_spawn_tick = Utc::now();
        }

        if recovered_players {
            self.last_player_recover_tick = Utc::now();
        }

        if recovered_npcs {
            self.last_npc_recover_tick = Utc::now();
        }

        if triggered_quakes {
            self.last_quake_tick = Utc::now();
        }
    }
}
