use eo::{data::{EOShort, EOChar}, protocol::{server::range, CharacterMapInfo}};
use tokio::sync::oneshot;

use super::Map;

impl Map {
    pub fn get_map_info(
        &self,
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
        respond_to: oneshot::Sender<range::Reply>,
    ) {
        let mut reply = range::Reply::default();
        if !player_ids.is_empty() {
            for player_id in player_ids {
                if let Some(character) = self.characters.get(&player_id) {
                    if !reply
                        .nearby
                        .characters
                        .iter()
                        .any(|c: &CharacterMapInfo| c.id == player_id)
                    {
                        reply.nearby.num_characters += 1;
                        reply.nearby.characters.push(character.to_map_info());
                    }
                }
            }
        }

        if !npc_indexes.is_empty() {
            for npc_index in npc_indexes {
                if let Some(npc) = self.npcs.get(&npc_index) {
                    if npc.alive {
                        reply.nearby.npcs.push(npc.to_map_info(&npc_index));
                    }
                }
            }
        }

        let _ = respond_to.send(reply);
    }
}