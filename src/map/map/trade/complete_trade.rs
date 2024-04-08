use std::cmp;

use eolib::protocol::{
    net::{
        server::{TradeItemData, TradeUseServerPacket},
        PacketAction, PacketFamily,
    },
    Emote,
};

use crate::{SETTINGS};

use super::super::Map;

impl Map {
    pub async fn complete_trade(&mut self, player_id: i32, partner_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let trade_items = character.trade_items.clone();

        let partner_character = match self.characters.get(&partner_id) {
            Some(partner_character) => partner_character,
            None => return,
        };

        let partner_trade_items = partner_character.trade_items.clone();

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.trade_items.clear();
        for item in &trade_items {
            character.remove_item(item.id, item.amount);
        }

        let character = match self.characters.get_mut(&partner_id) {
            Some(character) => character,
            None => return,
        };

        for item in &trade_items {
            let amount = cmp::min(
                SETTINGS.limits.max_item - character.get_item_amount(item.id),
                item.amount,
            );
            character.add_item(item.id, amount);
        }

        let character = match self.characters.get_mut(&partner_id) {
            Some(character) => character,
            None => return,
        };

        character.trade_items.clear();
        for item in &partner_trade_items {
            character.remove_item(item.id, item.amount);
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        for item in &partner_trade_items {
            let amount = cmp::min(
                SETTINGS.limits.max_item - character.get_item_amount(item.id),
                item.amount,
            );
            character.add_item(item.id, amount);
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let partner_character = match self.characters.get(&partner_id) {
            Some(character) => character,
            None => return,
        };

        if let Some(player) = character.player.as_ref() {
            player.set_trading(false);
            player.set_trade_accepted(false);

            player.send(
                PacketAction::Use,
                PacketFamily::Trade,
                &TradeUseServerPacket {
                    trade_data: TradeItemData {
                        partner_player_id: partner_id,
                        partner_items: partner_trade_items.clone(),
                        your_player_id: player_id,
                        your_items: trade_items.clone(),
                    },
                },
            );
        }

        if let Some(partner) = partner_character.player.as_ref() {
            partner.set_trading(false);
            partner.set_trade_accepted(false);

            partner.send(
                PacketAction::Use,
                PacketFamily::Trade,
                &TradeUseServerPacket {
                    trade_data: TradeItemData {
                        partner_player_id: player_id,
                        partner_items: trade_items.clone(),
                        your_player_id: partner_id,
                        your_items: partner_trade_items.clone(),
                    },
                },
            );
        }

        self.emote(player_id, Emote::Trade);
        self.emote(partner_id, Emote::Trade);
    }
}
