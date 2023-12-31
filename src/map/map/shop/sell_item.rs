use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{ShopSellServerPacket, ShopSoldItem},
            Item, PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn sell_item(&mut self, player_id: i32, item: Item, session_id: i32) {
        if item.amount == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.player.as_ref().unwrap().is_trading().await {
            return;
        }

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
            .find(|trade| trade.item_id == item.id && trade.sell_price > 0)
        {
            Some(trade) => trade,
            None => return,
        };

        let amount = cmp::min(item.amount, character.get_item_amount(item.id));

        if amount == 0 {
            return;
        }

        let price = trade.sell_price * amount;

        character.remove_item(item.id, amount);
        character.add_item(1, price);

        let reply = ShopSellServerPacket {
            gold_amount: character.get_item_amount(1),
            sold_item: ShopSoldItem {
                id: item.id,
                amount: character.get_item_amount(item.id),
            },
            weight: character.get_weight(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize ShopSellServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Sell,
            PacketFamily::Shop,
            writer.to_byte_array(),
        );
    }
}
