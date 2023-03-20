use eo::{data::{EOShort, EOChar}, protocol::NearbyInfo};
use tokio::sync::oneshot;

use super::Map;

impl Map {
    pub fn get_nearby_info(
        &self,
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    ) {
        let target = self.characters.get(&target_player_id).unwrap();
        let mut nearby_items = Vec::new();
        let mut nearby_npcs = Vec::new();
        let mut nearby_characters = Vec::new();
        for item in self.items.iter() {
            if target.is_in_range(item.coords) {
                nearby_items.push(item.to_item_map_info());
            }
        }
        for (index, npc) in self.npcs.iter() {
            if npc.alive && target.is_in_range(npc.coords) {
                nearby_npcs.push(npc.to_map_info(index));
            }
        }
        for character in self.characters.values() {
            if target_player_id == character.player_id.unwrap()
                || target.is_in_range(character.coords)
            {
                nearby_characters.push(character.to_map_info());
            }
        }
        let _ = respond_to.send(NearbyInfo {
            num_characters: nearby_characters.len() as EOChar,
            items: nearby_items,
            npcs: nearby_npcs,
            characters: nearby_characters,
        });
    }
}