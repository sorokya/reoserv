use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::ShopBuyServerPacket, Item, PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SETTINGS, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn buy_item(&mut self, player_id: i32, item: Item, session_id: i32) {
        if item.amount <= 0 || item.amount > SETTINGS.limits.max_item {
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

        if npc_data.r#type != NpcType::Shop {
            return;
        }

        let shop = match SHOP_DB
            .shops
            .iter()
            .find(|shop| shop.behavior_id == npc_data.behavior_id)
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

        let amount = cmp::min(amount, trade.max_amount);

        let price = trade.buy_price * amount;

        if character.get_item_amount(1) < price {
            return;
        }

        character.remove_item(1, price);
        character.add_item(item.id, amount);

        let weight = character.get_weight();

        let reply = ShopBuyServerPacket {
            gold_amount: character.get_item_amount(1),
            bought_item: Item {
                id: item.id,
                amount,
            },
            weight,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize ShopBuyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Buy,
            PacketFamily::Shop,
            writer.to_byte_array(),
        );
    }
}
