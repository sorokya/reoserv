use eolib::protocol::net::Item;

use crate::map::{MapState, MapStateCharacter, MapStateChest, MapStateItem, MapStateNpc};

use super::super::Map;

impl Map {
    pub fn get_state(&self) -> MapState {
        MapState {
            name: self.file.name.to_owned(),
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
                    alive: npc.alive,
                })
                .collect::<Vec<_>>(),
            characters: self
                .characters
                .iter()
                .map(|(_, character)| MapStateCharacter {
                    id: character.id,
                    name: character.name.clone(),
                    coords: character.coords,
                    level: character.level,
                    class: character.class,
                    guild: match character.guild_tag.as_ref() {
                        Some(tag) => tag.to_owned(),
                        None => "".to_string(),
                    },
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
