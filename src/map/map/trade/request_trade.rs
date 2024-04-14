use eolib::protocol::net::{server::TradeRequestServerPacket, PacketAction, PacketFamily};

use crate::{utils::in_client_range, SETTINGS};

use super::super::Map;

impl Map {
    pub fn request_trade(&self, player_id: i32, target_player_id: i32) {
        if self.id == SETTINGS.jail.map {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if target.hidden || target.captcha_open {
            return;
        }

        let target_player = match target.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        if in_client_range(&character.coords, &target.coords) {
            player.set_interact_player_id(Some(target_player_id));

            target_player.send(
                PacketAction::Request,
                PacketFamily::Trade,
                &TradeRequestServerPacket {
                    partner_player_id: player_id,
                    partner_player_name: character.name.clone(),
                },
            );
        }
    }
}
