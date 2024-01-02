use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{NearbyInfo, RefreshReplyServerPacket},
        PacketAction, PacketFamily,
    },
};

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub fn request_refresh(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let packet = RefreshReplyServerPacket {
            nearby: NearbyInfo {
                characters: self
                    .characters
                    .iter()
                    .filter_map(|(_, other)| {
                        if !other.hidden && in_client_range(&character.coords, &other.coords) {
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
                ..Default::default()
            },
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing RefreshReplyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Refresh,
            writer.to_byte_array(),
        );
    }
}