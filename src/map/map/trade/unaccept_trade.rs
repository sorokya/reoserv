use eolib::{
    data::EoWriter,
    protocol::net::{PacketAction, PacketFamily},
};

use super::super::Map;

const UNAGREED: i32 = 0;

impl Map {
    pub async fn unaccept_trade(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let partner_id = match player.get_interact_player_id().await {
            Some(partner_id) => partner_id,
            None => return,
        };

        let partner_character = match self.characters.get(&partner_id) {
            Some(partner_character) => partner_character,
            None => return,
        };

        let partner = match partner_character.player.as_ref() {
            Some(partner) => partner,
            None => return,
        };

        player.set_trade_accepted(false);

        let mut writer = EoWriter::new();
        writer.add_char(UNAGREED);

        player.send(
            PacketAction::Spec,
            PacketFamily::Trade,
            writer.to_byte_array(),
        );

        let mut writer = EoWriter::new();
        writer.add_short(player_id);
        writer.add_char(UNAGREED);
        partner.send(
            PacketAction::Agree,
            PacketFamily::Trade,
            writer.to_byte_array(),
        );
    }
}
