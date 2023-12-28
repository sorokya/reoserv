use chrono::Utc;
use eo::{
    data::{EOInt, i32},
    protocol::Coords,
    pubs::{EmfFile, EmfTileSpec},
};

use crate::SETTINGS;

use crate::map::chest::{Chest, ChestSpawn};

pub fn create_chests(map_id: i32, file: &EmfFile) -> Vec<Chest> {
    let mut chests: Vec<Chest> = Vec::new();
    let now = Utc::now();
    for item in &file.items {
        let item_coords = Coords {
            x: item.x,
            y: item.y,
        };

        if file
            .spec_rows
            .iter()
            .find(|row| row.y == item.y)
            .and_then(|row| {
                row.tiles
                    .iter()
                    .find(|tile| tile.spec == EmfTileSpec::Chest && tile.x == item.x)
            })
            .is_none()
        {
            continue;
        }

        if let Some(chest) = chests.iter_mut().find(|chest| chest.coords == item_coords) {
            if chest.key.is_none() && item.key_required != 0 {
                chest.key = Some(item.key_required);
            }

            if item.chest_slot as EOInt + 1 > SETTINGS.chest.slots {
                warn!(
                    "Chest at map {} ({:?}) has too many slots",
                    map_id, item_coords
                );
                continue;
            }

            chest.spawns.push(ChestSpawn {
                slot: item.chest_slot + 1,
                item_id: item.item_id,
                amount: item.item_amount,
                spawn_time: item.spawn_time,
                last_taken: now,
            });
        } else {
            chests.push(Chest {
                coords: item_coords,
                items: Vec::new(),
                spawns: vec![ChestSpawn {
                    slot: item.chest_slot + 1,
                    item_id: item.item_id,
                    amount: item.item_amount,
                    spawn_time: item.spawn_time,
                    last_taken: now,
                }],
                key: match item.key_required {
                    0 => None,
                    key => Some(key),
                },
            });
        }
    }

    // For any chests that don't have spawns
    for row in &file.spec_rows {
        for tile in &row.tiles {
            if tile.spec == EmfTileSpec::Chest
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
