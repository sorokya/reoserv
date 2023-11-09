use crate::SETTINGS;

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
        let mut triggered_spikes = false;

        self.npc_spawn_ticks += 1;
        self.item_spawn_ticks += 1;
        self.player_recover_ticks += 1;
        self.npc_recover_ticks += 1;
        self.quake_ticks += 1;
        self.spike_ticks += 1;

        for map in maps {
            map.act_npcs();

            if self.npc_spawn_ticks >= SETTINGS.npcs.respawn_rate {
                map.spawn_npcs();
                spawned_npcs = true;
            }

            if self.item_spawn_ticks >= SETTINGS.world.chest_spawn_rate {
                map.spawn_items();
                spawned_items = true;
            }

            if self.player_recover_ticks >= SETTINGS.world.recover_rate {
                map.recover_players();
                recovered_players = true;
            }

            if self.npc_recover_ticks >= SETTINGS.world.npc_recover_rate {
                map.recover_npcs();
                recovered_npcs = true;
            }

            if self.quake_ticks >= SETTINGS.map.quake_rate {
                map.timed_quake();
                triggered_quakes = true;
            }

            if self.spike_ticks >= SETTINGS.map.spike_rate {
                map.timed_spikes();
                triggered_spikes = true;
            }
        }

        if spawned_npcs {
            self.npc_spawn_ticks = 0;
        }

        if spawned_items {
            self.item_spawn_ticks = 0;
        }

        if recovered_players {
            self.player_recover_ticks = 0;
        }

        if recovered_npcs {
            self.npc_recover_ticks = 0;
        }

        if triggered_quakes {
            self.quake_ticks = 0;
        }

        if triggered_spikes {
            self.spike_ticks = 0;
        }
    }
}
