use eolib::protocol::{
    net::{
        server::{PriestOpenServerPacket, PriestReply, PriestReplyServerPacket},
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{
    map::WeddingState,
    utils::{dressed_for_wedding, in_client_range},
    NPC_DB, SETTINGS,
};

use super::super::Map;

impl Map {
    pub fn open_priest(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.fiance.is_none() || character.partner.is_some() {
            return;
        }

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        if let Some(wedding) = &self.wedding {
            if !matches!(wedding.state, WeddingState::Requested | WeddingState::Done) {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Priest,
                    &PriestReplyServerPacket {
                        reply_code: PriestReply::Busy,
                    },
                );
                return;
            }
        }

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Priest {
            return;
        }

        if !in_client_range(&character.coords, &npc.coords) {
            return;
        }

        if character.level < SETTINGS.marriage.min_level {
            player.send(
                PacketAction::Reply,
                PacketFamily::Priest,
                &PriestReplyServerPacket {
                    reply_code: PriestReply::LowLevel,
                },
            );
            return;
        }

        if !dressed_for_wedding(&character) {
            player.send(
                PacketAction::Reply,
                PacketFamily::Priest,
                &PriestReplyServerPacket {
                    reply_code: PriestReply::NotDressed,
                },
            );
            return;
        }

        player.set_interact_npc_index(npc_index);
        player.send(
            PacketAction::Open,
            PacketFamily::Priest,
            &PriestOpenServerPacket { session_id },
        );
    }
}
