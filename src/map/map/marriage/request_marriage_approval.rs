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

use crate::{NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn request_marriage_approval(&mut self, player_id: i32, npc_index: i32, name: String) {
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

        if character.partner.is_some() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Marriage,
                &MarriageReplyServerPacket {
                    reply_code: MarriageReply::AlreadyMarried,
                    reply_code_data: None,
                },
            );
            return;
        }

        if character.get_item_amount(1) < SETTINGS.marriage.approval_cost {
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

        character.remove_item(1, SETTINGS.marriage.approval_cost);
        character.fiance = Some(name);

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
    }
}
