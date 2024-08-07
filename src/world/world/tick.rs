use crate::SETTINGS;

use super::World;

const ONE_SECOND: i32 = 8;

impl World {
    pub async fn tick(&mut self) {
        let maps = match self.maps {
            Some(ref maps) => maps.values(),
            None => return,
        };

        self.player_ticks += 1;
        self.npc_act_ticks += 1;
        self.npc_spawn_ticks += 1;
        self.item_spawn_ticks += 1;
        self.player_recover_ticks += 1;
        self.npc_recover_ticks += 1;
        self.quake_ticks += 1;
        self.spike_ticks += 1;
        self.drain_ticks += 1;
        self.warp_suck_ticks += 1;
        self.arena_ticks += 1;
        self.door_close_ticks += 1;
        self.wedding_ticks += 1;
        self.evacuate_ticks += 1;
        self.jukebox_ticks += 1;
        self.drop_ticks += 1;

        if self.player_ticks >= ONE_SECOND {
            for player in self.players.values() {
                player.tick();
            }
            self.player_ticks = 0;
        }

        for map in maps {
            if self.npc_act_ticks >= SETTINGS.npcs.act_rate {
                map.act_npcs();
            }

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

            if self.quake_ticks >= SETTINGS.world.quake_rate {
                map.timed_quake();
            }

            if self.spike_ticks >= SETTINGS.world.spike_rate {
                map.timed_spikes();
            }

            if self.drain_ticks >= SETTINGS.world.drain_rate {
                map.timed_drain();
            }

            if self.warp_suck_ticks >= ONE_SECOND {
                map.timed_warp_suck();
            }

            if self.door_close_ticks >= ONE_SECOND {
                map.timed_door_close();
            }

            if self.wedding_ticks >= ONE_SECOND {
                map.timed_wedding();
            }

            if self.evacuate_ticks >= ONE_SECOND {
                map.timed_evacuate();
            }

            if self.arena_ticks >= ONE_SECOND {
                map.timed_arena();
            }

            if self.jukebox_ticks >= ONE_SECOND && SETTINGS.jukebox.track_timer > 0 {
                map.jukebox_timer();
            }

            if self.drop_ticks >= ONE_SECOND {
                map.timed_drop_protection();
            }
        }

        if self.player_ticks >= ONE_SECOND {
            self.player_ticks = 0;
        }

        if self.npc_act_ticks >= SETTINGS.npcs.act_rate {
            self.npc_act_ticks = 0;
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

        if self.quake_ticks >= SETTINGS.world.quake_rate {
            self.quake_ticks = 0;
        }

        if self.spike_ticks >= SETTINGS.world.spike_rate {
            self.spike_ticks = 0;
        }

        if self.drain_ticks >= SETTINGS.world.drain_rate {
            self.drain_ticks = 0;
        }

        if self.warp_suck_ticks >= ONE_SECOND {
            self.warp_suck_ticks = 0;
        }

        if self.door_close_ticks >= ONE_SECOND {
            self.door_close_ticks = 0;
        }

        if self.arena_ticks >= ONE_SECOND {
            self.arena_ticks = 0;
        }

        if self.jukebox_ticks >= ONE_SECOND {
            self.jukebox_ticks = 0;
        }

        if self.drop_ticks >= ONE_SECOND {
            self.drop_ticks = 0;
        }

        if self.wedding_ticks >= ONE_SECOND {
            self.wedding_ticks = 0;
        }

        if self.evacuate_ticks >= ONE_SECOND {
            self.evacuate_ticks = 0;
        }
    }
}
