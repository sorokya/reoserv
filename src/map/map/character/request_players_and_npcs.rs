use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{NearbyInfo, RangeReplyServerPacket},
        PacketAction, PacketFamily,
    },
};

use super::super::Map;

impl Map {
    pub fn request_players_and_npcs(
        &self,
        player_id: i32,
        player_ids: Vec<i32>,
        npc_indexes: Vec<i32>,
    ) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let packet = RangeReplyServerPacket {
            nearby: NearbyInfo {
                characters: self
                    .characters
                    .iter()
                    .filter_map(|(index, character)| {
                        if player_ids.contains(index) {
                            Some(character.to_map_info())
                        } else {
                            None
                        }
                    })
                    .collect(),
                npcs: self
                    .npcs
                    .iter()
                    .filter_map(|(index, npc)| {
                        if npc_indexes.contains(index) {
                            Some(npc.to_map_info(index))
                        } else {
                            None
                        }
                    })
                    .collect(),
                ..Default::default()
            },
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing RangeReplyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Range,
            writer.to_byte_array(),
        );
    }
}
