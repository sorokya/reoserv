use eolib::protocol::net::{
    server::{NearbyInfo, RefreshReplyServerPacket},
    PacketAction, PacketFamily,
};

use crate::{
    deep::{BossPingServerPacket, FAMILY_BOSS},
    utils::in_client_range,
};

use super::super::Map;

impl Map {
    pub fn request_refresh(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        player.send(
            PacketAction::Reply,
            PacketFamily::Refresh,
            &RefreshReplyServerPacket {
                nearby: NearbyInfo {
                    characters: self
                        .characters
                        .iter()
                        .filter_map(|(id, other)| {
                            if (!other.hidden || *id == player_id)
                                && in_client_range(&character.coords, &other.coords)
                            {
                                Some(other.to_map_info())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    npcs: self
                        .npcs
                        .iter()
                        .filter_map(|(index, npc)| {
                            if npc.alive && in_client_range(&character.coords, &npc.coords) {
                                Some(npc.to_map_info(index))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    items: self
                        .items
                        .iter()
                        .filter_map(|(index, item)| {
                            if in_client_range(&character.coords, &item.coords) {
                                Some(item.to_map_info(index))
                            } else {
                                None
                            }
                        })
                        .collect(),
                },
            },
        );

        if character.is_deep {
            for (npc_index, npc) in self.npcs.iter().filter(|(_, npc)| {
                npc.alive && npc.boss && in_client_range(&character.coords, &npc.coords)
            }) {
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
