use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{ShopCraftItem, ShopOpenServerPacket, ShopTradeItem},
            CharItem, PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn open_shop(&mut self, player_id: i32, npc_index: i32) {
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

        player.set_interact_npc_index(npc_index);

        // TODO: stupid that I have to map over the shop data here
        // they should be compatible in protocol
        let reply = ShopOpenServerPacket {
            session_id,
            shop_name: shop.name.clone(),
            trade_items: shop
                .trades
                .iter()
                .map(|trade| ShopTradeItem {
                    item_id: trade.item_id,
                    buy_price: trade.buy_price,
                    sell_price: trade.sell_price,
                    max_buy_amount: trade.max_amount,
                })
                .collect(),
            craft_items: shop
                .crafts
                .iter()
                .map(|craft| ShopCraftItem {
                    item_id: craft.item_id,
                    ingredients: [
                        CharItem {
                            id: craft.ingredients[0].item_id,
                            amount: craft.ingredients[0].amount,
                        },
                        CharItem {
                            id: craft.ingredients[1].item_id,
                            amount: craft.ingredients[1].amount,
                        },
                        CharItem {
                            id: craft.ingredients[2].item_id,
                            amount: craft.ingredients[2].amount,
                        },
                        CharItem {
                            id: craft.ingredients[3].item_id,
                            amount: craft.ingredients[3].amount,
                        },
                    ],
                })
                .collect(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize ShopOpenServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Open,
            PacketFamily::Shop,
            writer.to_byte_array(),
        );
    }
}
