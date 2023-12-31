use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{PacketAction, PacketFamily},
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

            let mut writer = EoWriter::new();
            let number_of_characters = 1;
            writer.add_char(number_of_characters);
            writer.add_byte(0xff);

            let map_info = character.to_map_info();
            map_info.serialize(&mut writer);

            self.send_buf_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                writer.to_byte_array(),
            );

            let mut writer = EoWriter::new();
            writer.add_short(player_id);
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

            let mut writer = EoWriter::new();
            writer.add_short(player_id);

            self.send_buf_near(
                &character.coords,
                PacketAction::Remove,
                PacketFamily::AdminInteract,
                writer.to_byte_array(),
            );
        }
    }
}
