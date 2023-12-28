use eolib::protocol::net::server::{RangeReplyServerPacket, CharacterMapInfo};
use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub fn get_map_info(
        &self,
        player_ids: Vec<i32>,
        npc_indexes: Vec<i32>,
        respond_to: oneshot::Sender<RangeReplyServerPacket>,
    ) {
        let mut reply = RangeReplyServerPacket::default();
        if !player_ids.is_empty() {
            for player_id in player_ids {
                if let Some(character) = self.characters.get(&player_id) {
                    if !reply
                        .nearby
                        .characters
                        .iter()
                        .any(|c: &CharacterMapInfo| c.player_id == player_id)
                    {
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
