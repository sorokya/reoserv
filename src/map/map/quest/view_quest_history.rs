use eolib::protocol::net::{
    server::{
        QuestListServerPacket, QuestListServerPacketPageData, QuestListServerPacketPageDataHistory,
    },
    PacketAction, PacketFamily, QuestPage,
};

use crate::QUEST_DB;

use super::super::Map;

impl Map {
    pub fn view_quest_history(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        let completed_quests = character
            .quests
            .iter()
            .filter_map(|q| {
                if q.done {
                    match QUEST_DB.get(&q.id) {
                        Some(quest) => Some(quest.name.to_owned()),
                        None => None,
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        player.send(
            PacketAction::List,
            PacketFamily::Quest,
            &QuestListServerPacket {
                page: QuestPage::History,
                quests_count: completed_quests.len() as i32,
                page_data: Some(QuestListServerPacketPageData::History(
                    QuestListServerPacketPageDataHistory { completed_quests },
                )),
            },
        )
    }
}
