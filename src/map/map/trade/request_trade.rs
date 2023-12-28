use eolib::{data::EoWriter, protocol::net::{PacketAction, PacketFamily}};

use crate::{utils::in_client_range, SETTINGS};

use super::super::Map;

const MAGIC_NUMBER: i32 = 138;

impl Map {
    pub fn request_trade(&self, player_id: i32, target_player_id: i32) {
        if self.id == SETTINGS.jail.map {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character");
                return;
            }
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => {
                error!("Failed to get player");
                return;
            }
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get target");
                return;
            }
        };

        let target_player = match target.player.as_ref() {
            Some(player) => player,
            None => {
                error!("Failed to get target player");
                return;
            }
        };

        if in_client_range(&character.coords, &target.coords) {
            player.set_interact_player_id(Some(target_player_id));

            let mut writer = EoWriter::new();
            writer.add_char(MAGIC_NUMBER);
            writer.add_short(player_id);
            writer.add_string(&character.name);
            target_player.send(PacketAction::Request, PacketFamily::Trade, writer.to_byte_array());
        }
    }
}
