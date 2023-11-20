use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::shop::Buy, Item, PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::{NPC_DB, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn buy_item(&mut self, player_id: EOShort, item: Item, session_id: EOShort) {
        if item.amount == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let actual_session_id = match character.player.as_ref().unwrap().get_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get session id {}", e);
                return;
            }
        };

        if actual_session_id != session_id {
            return;
        }

        let npc_index = match character
            .player
            .as_ref()
            .unwrap()
            .get_interact_npc_index()
            .await
        {
            Some(index) => index,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != EnfNpcType::Shop {
            return;
        }

        let shop = match SHOP_DB
            .shops
            .iter()
            .find(|shop| shop.vendor_id == npc_data.behavior_id)
        {
            Some(shop) => shop,
            None => return,
        };

        let trade = match shop
            .trades
            .iter()
            .find(|trade| trade.item_id == item.id && trade.buy_price > 0)
        {
            Some(trade) => trade,
            None => return,
        };

        let amount = character.can_hold(item.id, item.amount);

        if amount == 0 {
            return;
        }

        let price = trade.buy_price * amount;

        if character.get_item_amount(1) < price {
            return;
        }

        character.remove_item(1, price);
        character.add_item(item.id, amount);

        let weight = character.get_weight();

        let reply = Buy {
            gold_amount: character.get_item_amount(1),
            bought_item: Item {
                id: item.id,
                amount,
            },
            weight,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Buy,
            PacketFamily::Shop,
            builder.get(),
        );
    }
}
