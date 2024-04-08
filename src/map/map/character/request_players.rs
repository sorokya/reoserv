use eolib::protocol::net::{
    server::{NearbyInfo, RangeReplyServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn request_players(&self, player_id: i32, player_ids: Vec<i32>) {
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
            PacketFamily::Range,
            &RangeReplyServerPacket {
                nearby: NearbyInfo {
                    characters: self
                        .characters
                        .iter()
                        .filter_map(|(index, character)| {
                            if !character.hidden && player_ids.contains(index) {
                                Some(character.to_map_info())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    ..Default::default()
                },
            },
        );
    }
}
