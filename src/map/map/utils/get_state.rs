use eolib::protocol::net::Item;

use crate::map::{MapState, MapStateCharacter, MapStateChest, MapStateItem, MapStateNpc};

use super::super::Map;

impl Map {
    pub fn get_state(&self) -> MapState {
        MapState {
            chests: self
                .chests
                .iter()
                .map(|chest| MapStateChest {
                    coords: chest.coords,
                    items: chest
                        .items
                        .iter()
                        .map(|item| Item {
                            id: item.item_id,
                            amount: item.amount,
                        })
                        .collect::<Vec<_>>(),
                })
                .collect::<Vec<_>>(),
            npcs: self
                .npcs
                .iter()
                .map(|(index, npc)| MapStateNpc {
                    index: *index,
                    id: npc.id,
                    coords: npc.coords,
                    hp: npc.hp,
                    alive: npc.alive,
                })
                .collect::<Vec<_>>(),
            characters: self
                .characters
                .iter()
                .map(|(id, character)| MapStateCharacter {
                    id: *id,
                    name: character.name.clone(),
                    coords: character.coords,
                    hp: character.hp,
                    tp: character.tp,
                    level: character.level,
                })
                .collect::<Vec<_>>(),
            items: self
                .items
                .iter()
                .map(|(index, item)| MapStateItem {
                    index: *index,
                    coords: item.coords,
                    id: item.id,
                    amount: item.amount,
                })
                .collect::<Vec<_>>(),
        }
    }
}
