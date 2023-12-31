use chrono::Utc;
use eolib::protocol::map::{Emf, MapTileSpec};
use eolib::protocol::Coords;

use crate::SETTINGS;

use crate::map::chest::{Chest, ChestSpawn};

pub fn create_chests(map_id: i32, file: &Emf) -> Vec<Chest> {
    let mut chests: Vec<Chest> = Vec::new();
    let now = Utc::now();
    for item in &file.items {
        if file
            .tile_spec_rows
            .iter()
            .find(|row| row.y == item.coords.y)
            .and_then(|row| {
                row.tiles
                    .iter()
                    .find(|tile| tile.tile_spec == MapTileSpec::Chest && tile.x == item.coords.x)
            })
            .is_none()
        {
            continue;
        }

        if let Some(chest) = chests.iter_mut().find(|chest| chest.coords == item.coords) {
            if chest.key.is_none() && item.key != 0 {
                chest.key = Some(item.key);
            }

            if item.chest_slot + 1 > SETTINGS.chest.slots {
                warn!(
                    "Chest at map {} ({:?}) has too many slots",
                    map_id, item.coords
                );
                continue;
            }

            chest.spawns.push(ChestSpawn {
                slot: item.chest_slot + 1,
                item_id: item.item_id,
                amount: item.amount,
                spawn_time: item.spawn_time,
                last_taken: now,
            });
        } else {
            chests.push(Chest {
                coords: item.coords,
                items: Vec::new(),
                spawns: vec![ChestSpawn {
                    slot: item.chest_slot + 1,
                    item_id: item.item_id,
                    amount: item.amount,
                    spawn_time: item.spawn_time,
                    last_taken: now,
                }],
                key: match item.key {
                    0 => None,
                    key => Some(key),
                },
            });
        }
    }

    // For any chests that don't have spawns
    for row in &file.tile_spec_rows {
        for tile in &row.tiles {
            if tile.tile_spec == MapTileSpec::Chest
                && !chests
                    .iter()
                    .any(|chest| chest.coords.y == row.y && chest.coords.x == tile.x)
            {
                chests.push(Chest {
                    coords: Coords {
                        x: tile.x,
                        y: row.y,
                    },
                    items: Vec::new(),
                    spawns: Vec::new(),
                    key: None,
                });
            }
        }
    }

    chests
}
