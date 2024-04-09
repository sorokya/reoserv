use eolib::protocol::net::server::NearbyInfo;
use tokio::sync::oneshot;

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub fn get_nearby_info(&self, player_id: i32, respond_to: oneshot::Sender<NearbyInfo>) {
        let target = self.characters.get(&player_id).unwrap();
        let mut nearby_items = Vec::new();
        let mut nearby_npcs = Vec::new();
        let mut nearby_characters = Vec::new();
        for (index, item) in self.items.iter() {
            if in_client_range(&target.coords, &item.coords) {
                nearby_items.push(item.to_map_info(index));
            }
        }
        for (index, npc) in self.npcs.iter() {
            if npc.alive && in_client_range(&target.coords, &npc.coords) {
                nearby_npcs.push(npc.to_map_info(index));
            }
        }
        for character in self.characters.values() {
            if player_id == character.player_id.unwrap()
                || (!character.hidden && in_client_range(&target.coords, &character.coords))
            {
                nearby_characters.push(character.to_map_info());
            }
        }
        let _ = respond_to.send(NearbyInfo {
            items: nearby_items,
            npcs: nearby_npcs,
            characters: nearby_characters,
        });
    }
}
