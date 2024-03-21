use eolib::protocol::{
    net::{
        server::{
            MarriageReply, MarriageReplyServerPacket, MarriageReplyServerPacketReplyCodeData,
            MarriageReplyServerPacketReplyCodeDataSuccess,
        },
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use crate::{NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn request_divorce(&mut self, player_id: i32, npc_index: i32, name: String) {
        match self.npcs.get(&npc_index) {
            Some(npc) => {
                let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                    Some(npc_data) => npc_data,
                    None => return,
                };

                if npc_data.r#type != NpcType::Lawyer {
                    return;
                }
            }
            None => return,
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player {
            Some(ref player) => player.to_owned(),
            None => return,
        };

        if let Some(error_code) = match character.partner.as_ref() {
            Some(actual_name) => {
                if *actual_name != name {
                    Some(MarriageReply::WrongName)
                } else {
                    None
                }
            }
            None => Some(MarriageReply::NotMarried),
        } {
            player.send(
                PacketAction::Reply,
                PacketFamily::Marriage,
                &MarriageReplyServerPacket {
                    reply_code: error_code,
                    reply_code_data: None,
                },
            );
            return;
        }

        if character.get_item_amount(1) < SETTINGS.marriage.divorce_cost {
            player.send(
                PacketAction::Reply,
                PacketFamily::Marriage,
                &MarriageReplyServerPacket {
                    reply_code: MarriageReply::NotEnoughGold,
                    reply_code_data: None,
                },
            );
            return;
        }

        character.remove_item(1, SETTINGS.marriage.divorce_cost);
        character.partner = None;

        player.send(
            PacketAction::Reply,
            PacketFamily::Marriage,
            &MarriageReplyServerPacket {
                reply_code: MarriageReply::Success,
                reply_code_data: Some(MarriageReplyServerPacketReplyCodeData::Success(
                    MarriageReplyServerPacketReplyCodeDataSuccess {
                        gold_amount: character.get_item_amount(1),
                    },
                )),
            },
        );

        if let Some((_, partner)) = self.characters.iter_mut().find(|(_, c)| c.name == name) {
            partner.partner = None;
            partner.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::Marriage,
                &MarriageReplyServerPacket {
                    reply_code: MarriageReply::DivorceNotification,
                    reply_code_data: None,
                },
            );

            return;
        }

        let world = self.world.clone();
        let pool = self.pool.clone();

        tokio::spawn(async move {
            if let Ok(character) = world.get_character_by_name(name.to_owned()).await {
                if let Ok(map) = world.get_map(character.map_id).await {
                    map.divorce_partner(character.player_id.unwrap());
                    return;
                }
            }

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get sql connection: {}", e);
                    return;
                }
            };

            if let Err(e) = conn
                .exec_drop(
                    include_str!("../../../sql/divorce_character.sql"),
                    params! {
                        "name" => name
                    },
                )
                .await
            {
                error!("Failed to divorce character: {}", e);
            }
        });
    }
}
