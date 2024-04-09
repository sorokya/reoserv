use std::cmp;

use eolib::protocol::{
    net::{
        server::{ShopSellServerPacket, ShopSoldItem},
        Item, PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{NPC_DB, SETTINGS, SHOP_DB};

use super::super::Map;

impl Map {
    pub fn sell_item(&mut self, player_id: i32, npc_index: i32, item: Item) {
        if item.amount <= 0 || item.amount > SETTINGS.limits.max_item {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
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

        let amount = cmp::min(amount, trade.max_amount);

        let price = cmp::min(trade.sell_price * amount, SETTINGS.limits.max_item);

        character.remove_item(item.id, amount);
        character.add_item(1, price);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Sell,
                PacketFamily::Shop,
                &ShopSellServerPacket {
                    gold_amount: character.get_item_amount(1),
                    sold_item: ShopSoldItem {
                        id: item.id,
                        amount: character.get_item_amount(item.id),
                    },
                    weight: character.get_weight(),
                },
            );
        }
    }
}
