use eo::{
    data::{EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{server::shop::Create, Item, PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::{NPC_DB, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn craft_item(&mut self, player_id: EOShort, item_id: EOShort, session_id: EOShort) {
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

        let craft = match shop.crafts.iter().find(|craft| craft.item_id == item_id) {
            Some(craft) => craft,
            None => return,
        };

        if craft.ingredient1_item_id > 0
            && character.get_item_amount(craft.ingredient1_item_id)
                < craft.ingredient1_amount as EOInt
        {
            return;
        }

        if craft.ingredient2_item_id > 0
            && character.get_item_amount(craft.ingredient2_item_id)
                < craft.ingredient2_amount as EOInt
        {
            return;
        }

        if craft.ingredient3_item_id > 0
            && character.get_item_amount(craft.ingredient3_item_id)
                < craft.ingredient3_amount as EOInt
        {
            return;
        }

        if craft.ingredient4_item_id > 0
            && character.get_item_amount(craft.ingredient4_item_id)
                < craft.ingredient4_amount as EOInt
        {
            return;
        }

        if craft.ingredient1_item_id > 0 {
            character.remove_item(craft.ingredient1_item_id, craft.ingredient1_amount as EOInt);
        }

        if craft.ingredient2_item_id > 0 {
            character.remove_item(craft.ingredient2_item_id, craft.ingredient2_amount as EOInt);
        }

        if craft.ingredient3_item_id > 0 {
            character.remove_item(craft.ingredient3_item_id, craft.ingredient3_amount as EOInt);
        }

        if craft.ingredient4_item_id > 0 {
            character.remove_item(craft.ingredient4_item_id, craft.ingredient4_amount as EOInt);
        }

        character.add_item(item_id, 1);

        let reply = Create {
            craft_item_id: item_id,
            weight: character.get_weight(),
            ingredients: [
                Item {
                    id: craft.ingredient1_item_id,
                    amount: character.get_item_amount(craft.ingredient1_item_id),
                },
                Item {
                    id: craft.ingredient2_item_id,
                    amount: character.get_item_amount(craft.ingredient2_item_id),
                },
                Item {
                    id: craft.ingredient3_item_id,
                    amount: character.get_item_amount(craft.ingredient3_item_id),
                },
                Item {
                    id: craft.ingredient4_item_id,
                    amount: character.get_item_amount(craft.ingredient4_item_id),
                },
            ],
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Create,
            PacketFamily::Shop,
            builder.get(),
        );
    }
}
