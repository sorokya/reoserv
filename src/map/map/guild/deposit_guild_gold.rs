use std::cmp;

use eolib::protocol::{
    net::{PacketAction, PacketFamily, server::GuildBuyServerPacket},
    r#pub::NpcType,
};

use crate::{NPC_DB, SETTINGS, db::insert_params};

use super::super::Map;

impl Map {
    pub fn deposit_guild_gold(&mut self, player_id: i32, npc_index: i32, amount: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let tag = match character.guild_tag.as_ref() {
            Some(tag) => tag.to_owned(),
            None => return,
        };

        match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => match NPC_DB.load().npcs.get(npc.id as usize - 1) {
                Some(npc_data) => {
                    if npc_data.r#type != NpcType::Guild {
                        return;
                    }
                }
                None => return,
            },
            None => return,
        }

        let amount = cmp::min(amount, character.get_item_amount(1));

        character.remove_item(1, amount);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Buy,
                PacketFamily::Guild,
                &GuildBuyServerPacket {
                    gold_amount: character.get_item_amount(1),
                },
            );
        }

        let db = self.db.clone();

        tokio::spawn(async move {
            let current_bank_amount = match db
                .query_int(&insert_params(
                    "SELECT `bank` FROM `guilds` WHERE `tag` = :tag",
                    &[("tag", &tag)],
                ))
                .await
            {
                Ok(Some(amount)) => amount,
                Ok(None) => return,
                Err(e) => {
                    tracing::error!("Error getting guild bank amount: {}", e);
                    return;
                }
            };

            if current_bank_amount >= SETTINGS.load().guild.bank_max_gold {
                return;
            }

            let amount = cmp::min(
                SETTINGS.load().guild.bank_max_gold - current_bank_amount,
                amount,
            );

            if let Err(e) = db
                .execute(&insert_params(
                    "UPDATE `guilds` SET `bank` = `bank` + :amount WHERE `tag` = :tag",
                    &[("amount", &amount), ("tag", &tag)],
                ))
                .await
            {
                tracing::error!("Error updating guild bank: {}", e);
            }
        });
    }
}
