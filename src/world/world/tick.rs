use crate::SETTINGS;

use super::World;

const ONE_SECOND: i32 = 8;

impl World {
    pub async fn tick(&mut self) {
        let maps = match self.maps {
            Some(ref maps) => maps.values(),
            None => return,
        };

        if SETTINGS.auto_pickup.enabled {
            self.auto_pickup_ticks += 1;
        }

        self.second_ticks += 1;
        self.npc_act_ticks += 1;
        self.item_spawn_ticks += 1;
        self.player_recover_ticks += 1;
        self.npc_recover_ticks += 1;
        self.quake_ticks += 1;
        self.spike_ticks += 1;
        self.drain_ticks += 1;

        if self.second_ticks >= ONE_SECOND {
            for player in self.players.values() {
                player.tick();
            }
        }

        if self.npc_act_ticks >= SETTINGS.npcs.act_rate {
            self.scripts.tick();
        }

        for map in maps {
            if self.npc_act_ticks >= SETTINGS.npcs.act_rate {
                map.act_npcs();
            }

            if self.auto_pickup_ticks >= SETTINGS.auto_pickup.rate && SETTINGS.auto_pickup.enabled {
                map.timed_auto_pickup();
            }

            if self.second_ticks >= ONE_SECOND {
                map.spawn_npcs();
                map.timed_warp_suck();
                map.timed_door_close();
                map.timed_wedding();
                map.timed_evacuate();
                map.timed_arena();
                if SETTINGS.jukebox.track_timer > 0 {
                    map.jukebox_timer();
                }
                map.timed_drop_protection();
                map.timed_ghost();
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

            if self.quake_ticks >= SETTINGS.world.quake_rate {
                map.timed_quake();
            }

            if self.spike_ticks >= SETTINGS.world.spike_rate {
                map.timed_spikes();
            }

            if self.drain_ticks >= SETTINGS.world.drain_rate {
                map.timed_drain();
            }
        }

        if self.second_ticks >= ONE_SECOND {
            self.second_ticks = 0;
        }

        if self.auto_pickup_ticks >= SETTINGS.auto_pickup.rate && SETTINGS.auto_pickup.enabled {
            self.auto_pickup_ticks = 0;
        }

        if self.npc_act_ticks >= SETTINGS.npcs.act_rate {
            self.npc_act_ticks = 0;
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

        if self.quake_ticks >= SETTINGS.world.quake_rate {
            self.quake_ticks = 0;
        }

        if self.spike_ticks >= SETTINGS.world.spike_rate {
            self.spike_ticks = 0;
        }

        if self.drain_ticks >= SETTINGS.world.drain_rate {
            self.drain_ticks = 0;
        }
    }
}
