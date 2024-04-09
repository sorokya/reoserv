use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            QuestAcceptClientPacket, QuestAcceptClientPacketReplyTypeData, QuestListClientPacket,
            QuestUseClientPacket,
        },
        PacketAction, QuestPage,
    },
};

use super::super::Player;

impl Player {
    fn quest_use(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let r#use = match QuestUseClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing QuestUseClientPacket: {}", e);
                    return;
                }
            };

            map.talk_to_quest_npc(self.id, r#use.npc_index, r#use.quest_id, session_id);
        }
    }

    fn quest_accept(&mut self, reader: EoReader) {
        let old_session_id = match self.session_id {
            Some(session_id) => session_id,
            None => return,
        };

        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let accept = match QuestAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing QuestAcceptClientPacket: {}", e);
                    return;
                }
            };

            if old_session_id != accept.session_id {
                return;
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => {
                    return;
                }
            };

            let action_id = match accept.reply_type_data {
                Some(QuestAcceptClientPacketReplyTypeData::Link(data)) => Some(data.action),
                _ => None,
            };

            map.reply_to_quest_npc(self.id, npc_index, accept.quest_id, session_id, action_id);
        }
    }

    fn quest_list(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let list = match QuestListClientPacket::deserialize(&reader) {
                Ok(list) => list,
                Err(e) => {
                    error!("Error deserializing QuestListClientPacket: {}", e);
                    return;
                }
            };

            match list.page {
                QuestPage::Progress => map.view_quest_progress(self.id),
                QuestPage::History => map.view_quest_history(self.id),
                _ => {}
            }
        }
    }

    pub fn handle_quest(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Use => self.quest_use(reader),
            PacketAction::Accept => self.quest_accept(reader),
            PacketAction::List => self.quest_list(reader),
            _ => error!("Unhandled packet Quest_{:?}", action),
        }
    }
}
