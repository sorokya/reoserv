use eolib::{data::EoWriter, protocol::net::{PacketAction, PacketFamily}};

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub async fn accept_trade_request(&mut self, player_id: i32, target_player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let target_player = match target_character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let target_player_interact_player_id = match target_player.get_interact_player_id().await {
            Some(player_id) => player_id,
            None => return,
        };

        if target_player_interact_player_id != player_id {
            return;
        }

        if !in_client_range(&character.coords, &target_character.coords) {
            return;
        }

        player.set_interact_player_id(Some(target_player_id));
        player.set_trading(true);
        target_player.set_trading(true);

        let mut writer = EoWriter::new();
        writer.add_short(target_player_id);
        writer.add_string(&target_character.name);
        writer.add_byte(0xff);
        writer.add_short(player_id);
        writer.add_string(&character.name);
        writer.add_byte(0xff);
        player.send(PacketAction::Open, PacketFamily::Trade, writer.to_byte_array());

        let mut writer = EoWriter::new();
        writer.add_short(player_id);
        writer.add_string(&character.name);
        writer.add_byte(0xff);
        writer.add_short(target_player_id);
        writer.add_string(&target_character.name);
        writer.add_byte(0xff);
        target_player.send(PacketAction::Open, PacketFamily::Trade, writer.to_byte_array());
    }
}
