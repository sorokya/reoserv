use std::cmp;

use eolib::protocol::{
    net::{Item, PacketAction, PacketFamily, server::ShopBuyServerPacket},
    r#pub::NpcType,
};

use crate::{NPC_DB, SETTINGS, SHOP_DB};

use super::super::Map;

impl Map {
    pub fn buy_item(&mut self, player_id: i32, npc_index: i32, item: Item) {
        if item.amount <= 0 || item.amount > SETTINGS.load().limits.max_item {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_db = NPC_DB.load();
        let npc_data = match npc_db.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Shop {
            return;
        }

        let shop_db = SHOP_DB.load();
        let shop = match shop_db
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

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Buy,
                PacketFamily::Shop,
                &ShopBuyServerPacket {
                    gold_amount: character.get_item_amount(1),
                    bought_item: Item {
                        id: item.id,
                        amount,
                    },
                    weight: character.get_weight(),
                },
            );
        }
    }
}
