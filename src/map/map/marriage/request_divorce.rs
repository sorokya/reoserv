use eolib::protocol::{
    net::{
        PacketAction, PacketFamily,
        server::{
            MarriageReply, MarriageReplyServerPacket, MarriageReplyServerPacketReplyCodeData,
            MarriageReplyServerPacketReplyCodeDataSuccess,
        },
    },
    r#pub::NpcType,
};

use crate::{NPC_DB, SETTINGS, db::insert_params};

use super::super::Map;

impl Map {
    pub fn request_divorce(&mut self, player_id: i32, npc_index: i32, name: String) {
        match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => {
                let npc_db = NPC_DB.load();
                let npc_data = match npc_db.npcs.get(npc.id as usize - 1) {
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

        if character.get_item_amount(1) < SETTINGS.load().marriage.divorce_cost {
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

        character.remove_item(1, SETTINGS.load().marriage.divorce_cost);
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
            if let Some(player) = partner.player.as_ref() {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Marriage,
                    &MarriageReplyServerPacket {
                        reply_code: MarriageReply::DivorceNotification,
                        reply_code_data: None,
                    },
                );
            }

            return;
        }

        let world = self.world.clone();
        let db = self.db.clone();

        tokio::spawn(async move {
            if let Ok(character) = world.get_character_by_name(&name).await
                && let Ok(map) = world.get_map(character.map_id).await
            {
                map.divorce_partner(character.player_id.unwrap());
                return;
            }

            if let Err(e) = db
                .execute(&insert_params(
                    include_str!("../../../sql/divorce_character.sql"),
                    &[("name", &name)],
                ))
                .await
            {
                tracing::error!("Failed to divorce character: {}", e);
            }
        });
    }
}
