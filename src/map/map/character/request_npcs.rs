use eolib::protocol::net::{server::NpcAgreeServerPacket, PacketAction, PacketFamily};

use crate::deep::{BossPingServerPacket, FAMILY_BOSS};

use super::super::Map;

impl Map {
    pub fn request_npcs(&self, player_id: i32, npc_indexes: Vec<i32>) {
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
                            if npc.alive && npc_indexes.contains(index) {
                                Some(npc.to_map_info(index))
                            } else {
                                None
                            }
                        })
                        .collect(),
                },
            );

            if character.is_deep {
                for (npc_index, npc) in self
                    .npcs
                    .iter()
                    .filter(|(index, npc)| npc.alive && npc.boss && npc_indexes.contains(index))
                {
                    player.send(
                        PacketAction::Ping,
                        PacketFamily::Unrecognized(FAMILY_BOSS),
                        &BossPingServerPacket {
                            npc_index: *npc_index,
                            npc_id: npc.id,
                            hp: npc.hp,
                            hp_percentage: npc.get_hp_percentage(),
                            killed: false,
                        },
                    );
                }
            }
        }
    }
}
