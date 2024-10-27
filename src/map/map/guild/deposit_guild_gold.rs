use std::cmp;

use eolib::protocol::{
    net::{server::GuildBuyServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};
use mysql_async::{prelude::Queryable, Params};
use mysql_common::{params, Row};

use crate::{NPC_DB, SETTINGS};

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

        match self.npcs.get(&npc_index) {
            Some(npc) => match NPC_DB.npcs.get(npc.id as usize - 1) {
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

        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let current_bank_amount = match conn
                .exec_first::<Row, &str, Params>(
                    "SELECT `bank` FROM Guild WHERE `tag` = :tag",
                    params! {
                        "tag" => &tag,
                    },
                )
                .await
            {
                Ok(Some(row)) => row.get::<i32, usize>(0).unwrap(),
                Ok(None) => return,
                Err(e) => {
                    error!("Error getting guild bank amount: {}", e);
                    return;
                }
            };

            if current_bank_amount >= SETTINGS.guild.bank_max_gold {
                return;
            }

            let amount = cmp::min(SETTINGS.guild.bank_max_gold - current_bank_amount, amount);

            if let Err(e) = conn
                .exec_drop(
                    "UPDATE Guild SET `bank` = `bank` + :amount WHERE `tag` = :tag",
                    params! {
                        "amount" => amount,
                        "tag" => tag,
                    },
                )
                .await
            {
                error!("Error updating guild bank: {}", e);
            }
        });
    }
}
