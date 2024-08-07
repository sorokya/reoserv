use eolib::protocol::{
    net::{
        server::{
            AdminInteractAgreeServerPacket, AdminInteractRemoveServerPacket, NearbyInfo,
            PlayersAgreeServerPacket,
        },
        PacketAction, PacketFamily,
    },
    AdminLevel,
};

use super::super::Map;

impl Map {
    pub fn toggle_hidden(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if i32::from(character.admin_level) < i32::from(AdminLevel::Guardian) {
            return;
        }

        if character.hidden {
            character.hidden = false;

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            self.send_packet_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                PlayersAgreeServerPacket {
                    nearby: NearbyInfo {
                        characters: vec![character.to_map_info()],
                        ..Default::default()
                    },
                },
            );

            self.send_packet_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::AdminInteract,
                AdminInteractAgreeServerPacket { player_id },
            );
        } else {
            character.hidden = true;

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            self.send_packet_near(
                &character.coords,
                PacketAction::Remove,
                PacketFamily::AdminInteract,
                AdminInteractRemoveServerPacket { player_id },
            );
        }
    }
}
