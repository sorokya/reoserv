use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::ShopCreateServerPacket, Item, PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SHOP_DB};

use super::super::Map;

impl Map {
    pub async fn craft_item(&mut self, player_id: i32, item_id: i32, session_id: i32) {
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

        let craft = match shop.crafts.iter().find(|craft| craft.item_id == item_id) {
            Some(craft) => craft,
            None => return,
        };

        for ingredient in craft.ingredients.iter() {
            if ingredient.item_id > 0
                && character.get_item_amount(ingredient.item_id) < ingredient.amount
            {
                return;
            }
        }

        for ingredient in craft.ingredients.iter() {
            if ingredient.item_id > 0 {
                character.remove_item(ingredient.item_id, ingredient.amount);
            }
        }

        character.add_item(item_id, 1);

        let reply = ShopCreateServerPacket {
            craft_item_id: item_id,
            weight: character.get_weight(),
            ingredients: [
                Item {
                    id: craft.ingredients[0].item_id,
                    amount: character.get_item_amount(craft.ingredients[0].item_id),
                },
                Item {
                    id: craft.ingredients[1].item_id,
                    amount: character.get_item_amount(craft.ingredients[1].item_id),
                },
                Item {
                    id: craft.ingredients[2].item_id,
                    amount: character.get_item_amount(craft.ingredients[2].item_id),
                },
                Item {
                    id: craft.ingredients[3].item_id,
                    amount: character.get_item_amount(craft.ingredients[3].item_id),
                },
            ],
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize ShopCreateServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Create,
            PacketFamily::Shop,
            writer.to_byte_array(),
        );
    }
}
