use crate::SETTINGS;

use super::World;

impl World {
    pub async fn tick(&mut self) {
        let maps = match self.maps {
            Some(ref maps) => maps.values(),
            None => return,
        };

        self.npc_spawn_ticks += 1;
        self.item_spawn_ticks += 1;
        self.player_recover_ticks += 1;
        self.npc_recover_ticks += 1;
        self.quake_ticks += 1;
        self.spike_ticks += 1;
        self.drain_ticks += 1;

        for map in maps {
            map.act_npcs();

            if self.npc_spawn_ticks >= SETTINGS.npcs.respawn_rate {
                map.spawn_npcs();
            }

            if self.item_spawn_ticks >= SETTINGS.world.chest_spawn_rate {
                map.spawn_items();
            }

            if self.player_recover_ticks >= SETTINGS.world.recover_rate {
                map.recover_players();
            }

            if self.npc_recover_ticks >= SETTINGS.world.npc_recover_rate {
                map.recover_npcs();
            }

            if self.quake_ticks >= SETTINGS.map.quake_rate {
                map.timed_quake();
            }

            if self.spike_ticks >= SETTINGS.map.spike_rate {
                map.timed_spikes();
            }

            if self.drain_ticks >= SETTINGS.map.drain_rate {
                map.timed_drain();
            }
        }

        if self.npc_spawn_ticks >= SETTINGS.npcs.respawn_rate {
            self.npc_spawn_ticks = 0;
        }

        if self.item_spawn_ticks >= SETTINGS.world.chest_spawn_rate {
            self.item_spawn_ticks = 0;
        }

        if self.player_recover_ticks >= SETTINGS.world.recover_rate {
            self.player_recover_ticks = 0;
        }

        if self.npc_recover_ticks >= SETTINGS.world.npc_recover_rate {
            self.npc_recover_ticks = 0;
        }

        if self.quake_ticks >= SETTINGS.map.quake_rate {
            self.quake_ticks = 0;
        }

        if self.spike_ticks >= SETTINGS.map.spike_rate {
            self.spike_ticks = 0;
        }

        if self.drain_ticks >= SETTINGS.map.drain_rate {
            self.drain_ticks = 0;
        }
    }
}
