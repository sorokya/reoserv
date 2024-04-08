use eolib::protocol::net::{server::NpcAgreeServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn request_npcs(&self, player_id: i32, npcs_indexes: Vec<i32>) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Agree,
                PacketFamily::Npc,
                &NpcAgreeServerPacket {
                    npcs: self
                        .npcs
                        .iter()
                        .filter_map(|(index, npc)| {
                            if npc.alive && npcs_indexes.contains(index) {
                                Some(npc.to_map_info(index))
                            } else {
                                None
                            }
                        })
                        .collect(),
                },
            );
        }
    }
}
