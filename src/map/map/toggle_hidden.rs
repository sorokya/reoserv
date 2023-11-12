use eo::{
    data::{EOShort, Serializeable, StreamBuilder, EO_BREAK_CHAR},
    protocol::{AdminLevel, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub fn toggle_hidden(&mut self, player_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.admin_level.to_char() < AdminLevel::Guardian.to_char() {
            return;
        }

        if character.hidden {
            character.hidden = false;

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            let mut builder = StreamBuilder::new();
            let number_of_characters = 1;
            builder.add_char(number_of_characters);
            builder.add_byte(EO_BREAK_CHAR);

            let map_info = character.to_map_info();
            map_info.serialize(&mut builder);

            self.send_buf_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                builder.get(),
            );

            let mut builder = StreamBuilder::new();
            builder.add_short(player_id);
            self.send_buf_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::AdminInteract,
                builder.get(),
            );
        } else {
            character.hidden = true;

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            let mut builder = StreamBuilder::new();
            builder.add_short(player_id);

            self.send_buf_near(
                &character.coords,
                PacketAction::Remove,
                PacketFamily::AdminInteract,
                builder.get(),
            );
        }
    }
}
