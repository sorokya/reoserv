use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{
                AdminInteractAgreeServerPacket, AdminInteractRemoveServerPacket, NearbyInfo,
                PlayersAgreeServerPacket,
            },
            PacketAction, PacketFamily,
        },
        AdminLevel,
    },
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

            let packet = PlayersAgreeServerPacket {
                nearby: NearbyInfo {
                    characters: vec![character.to_map_info()],
                    ..Default::default()
                },
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing PlayersAgreeServerPacket: {}", e);
                return;
            }

            self.send_buf_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                writer.to_byte_array(),
            );

            let packet = AdminInteractAgreeServerPacket { player_id };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing AdminInteractAgreeServerPacket: {}", e);
                return;
            }

            self.send_buf_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::AdminInteract,
                writer.to_byte_array(),
            );
        } else {
            character.hidden = true;

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            let packet = AdminInteractRemoveServerPacket { player_id };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing AdminInteractRemoveServerPacket: {}", e);
                return;
            }

            self.send_buf_near(
                &character.coords,
                PacketAction::Remove,
                PacketFamily::AdminInteract,
                writer.to_byte_array(),
            );
        }
    }
}
