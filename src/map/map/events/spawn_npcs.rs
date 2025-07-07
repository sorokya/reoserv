use std::cmp;

use eolib::protocol::{r#pub::NpcType, Coords, Direction};
use rand::Rng;

use crate::{map::NPCBuilder, NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn spawn_npcs(&mut self) {
        self.npcs.retain(|_, n| n.spawn_index.is_some() || n.alive);

        if self.file.npcs.is_empty() {
            return;
        }

        if self.npcs.is_empty() {
            let mut npc_index: i32 = 0;

            for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                let data_record = match NPC_DB.npcs.get(spawn.id as usize - 1) {
                    Some(npc) => npc,
                    None => {
                        error!(
                            "Failed to load NPC {} (Map: {}, Coords: {:?})",
                            spawn.id, self.id, spawn.coords,
                        );
                        continue;
                    }
                };

                for _ in 0..spawn.amount as i64 {
                    self.npcs.insert(
                        npc_index,
                        NPCBuilder::new()
                            .id(spawn.id)
                            .coords(Coords::default())
                            .direction(Direction::Down)
                            .spawn_index(spawn_index)
                            .spawn_type(spawn.spawn_type)
                            .spawn_time(spawn.spawn_time)
                            .alive(false)
                            .spawn_ticks(if SETTINGS.npcs.instant_spawn {
                                0
                            } else {
                                spawn.spawn_time
                            })
                            .hp(data_record.hp)
                            .max_hp(data_record.hp)
                            .boss(data_record.boss)
                            .child(data_record.child)
                            .build(),
                    );
                    npc_index += 1;
                }
            }
        }

        let mut rng = rand::thread_rng();
        let indexes = self.npcs.keys().cloned().collect::<Vec<i32>>();

        for index in indexes {
            let (child, alive, spawn_ticks, spawn_coords, spawn_type, npc_type, spawn_time) = {
                match self.npcs.get_mut(&index) {
                    Some(npc) => {
                        let spawn_index = match npc.spawn_index {
                            Some(index) => index,
                            None => continue,
                        };

                        npc.spawn_ticks = cmp::max(npc.spawn_ticks - 1, 0);

                        let spawn = &self.file.npcs[spawn_index];
                        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                            Some(npc_data) => npc_data,
                            None => continue,
                        };
                        (
                            npc.child,
                            npc.alive,
                            npc.spawn_ticks,
                            spawn.coords,
                            spawn.spawn_type,
                            npc_data.r#type,
                            spawn.spawn_time,
                        )
                    }
                    None => continue,
                }
            };

            if child {
                if let Some((_, boss)) = self.npcs.iter().find(|(_, n)| n.boss) {
                    if !boss.alive {
                        continue;
                    }
                }
            }

            if alive || spawn_ticks > 0 {
                continue;
            }

            let variable_coords =
                spawn_type != 7 && matches!(npc_type, NpcType::Passive | NpcType::Aggressive);

            let file_spawn_coords = spawn_coords;
            let mut spawn_coords = if !variable_coords {
                spawn_coords
            } else {
                Coords {
                    x: cmp::max(
                        cmp::min(spawn_coords.x + rng.gen_range(-2..=2), self.file.width),
                        0,
                    ) as i32,
                    y: cmp::max(
                        cmp::min(spawn_coords.y + rng.gen_range(-2..=2), self.file.height),
                        0,
                    ) as i32,
                }
            };

            let mut i = 0;
            while !self.is_tile_walkable_npc(&spawn_coords)
                && (i > 100 || !self.is_tile_occupied(&spawn_coords))
            {
                let x = cmp::max(
                    cmp::min(file_spawn_coords.x + rng.gen_range(-2..=2), self.file.width),
                    0,
                );
                let y = cmp::max(
                    cmp::min(
                        file_spawn_coords.y + rng.gen_range(-2..=2),
                        self.file.height,
                    ),
                    0,
                );
                spawn_coords = Coords {
                    x: x as i32,
                    y: y as i32,
                };

                i += 1;

                if i >= 200 {
                    break;
                }
            }

            let npc = match self.npcs.get_mut(&index) {
                Some(npc) => npc,
                None => continue,
            };

            npc.alive = true;
            npc.hp = npc.max_hp;
            npc.coords = spawn_coords;
            npc.direction = if spawn_type == 7 {
                Direction::from(spawn_time & 0x03)
            } else {
                match rand::random::<u8>() % 4 {
                    0 => Direction::Down,
                    1 => Direction::Left,
                    2 => Direction::Up,
                    3 => Direction::Right,
                    _ => unreachable!(),
                }
            };
        }
    }
}
