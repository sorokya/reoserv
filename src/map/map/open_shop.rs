use eo::{
    data::{EOShort, EOChar, StreamBuilder, Serializeable}, protocol::{server::shop, ShopTradeItem, ShopCraftItem, VeryShortItem, PacketAction, PacketFamily}, pubs::EnfNpcType,
};

use crate::{NPC_DB, SHOP_DB};

use super::Map;

impl Map {
    pub async fn open_shop(&mut self, player_id: EOShort, npc_index: EOChar) {
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

        let shop = match SHOP_DB.shops.iter().find(|shop| shop.vendor_id == npc_data.behavior_id) {
            Some(shop) => shop,
            None => return,
        };

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let session_id = match player.generate_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to generate session id {}", e);
                return;
            }
        };

        // TODO: stupid that I have to map over the shop data here
        // they should be compatible in protocol
        let reply = shop::Open {
            session_id,
            shop_name: shop.name.clone(),
            trade_items: shop.trades.iter().map(|trade| ShopTradeItem {
                item_id: trade.item_id,
                buy_price: trade.buy_price,
                sell_price: trade.sell_price,
                max_buy_amount: trade.max_amount,
            }).collect(),
            craft_items: shop.crafts.iter().map(|craft| ShopCraftItem {
                item_id: craft.item_id,
                ingredients: [
                    VeryShortItem {
                        id: craft.ingredient1_item_id,
                        amount: craft.ingredient1_amount,
                    },
                    VeryShortItem {
                        id: craft.ingredient2_item_id,
                        amount: craft.ingredient2_amount,
                    },
                    VeryShortItem {
                        id: craft.ingredient3_item_id,
                        amount: craft.ingredient3_amount,
                    },
                    VeryShortItem {
                        id: craft.ingredient4_item_id,
                        amount: craft.ingredient4_amount,
                    },
                ],
            }).collect(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Open, PacketFamily::Shop, builder.get());
    }
}
