use eolib::protocol::net::{
    server::{
        QuestListServerPacket, QuestListServerPacketPageData,
        QuestListServerPacketPageDataProgress, QuestProgressEntry, QuestRequirementIcon,
    },
    PacketAction, PacketFamily, QuestPage,
};
use eoplus::Arg;

use crate::QUEST_DB;

use super::super::Map;

impl Map {
    pub fn view_quest_progress(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        let quest_progress_entries = character
            .quests
            .iter()
            .filter_map(|q| {
                if q.done_at.is_some() && q.state != 0 {
                    return None;
                }

                let quest = match QUEST_DB.get(&q.id) {
                    Some(quest) => quest,
                    None => return None,
                };

                let state = match quest.states.get(q.state as usize) {
                    Some(state) => state,
                    None => return None,
                };

                let rule = match state.rules.iter().find(|rule| {
                    rule.name == "GotItems"
                        || rule.name == "KilledNpcs"
                        || rule.name == "KilledPlayers"
                        || rule.name == "TalkedToNpc"
                        || rule.name == "InputNpc"
                        || rule.name == "EnterCoord"
                        || rule.name == "EnterMap"
                        || rule.name == "LeaveMap"
                }) {
                    Some(rule) => rule,
                    None => return None,
                };

                let (icon, progress, target) = match rule.name.as_str() {
                    "TalkedToNpc" | "InputNpc" => (QuestRequirementIcon::Talk, 0, 0),
                    "GotItems" => (
                        QuestRequirementIcon::Item,
                        if let Arg::Int(item_id) = rule.args[0] {
                            character.get_item_amount(item_id)
                        } else {
                            0
                        },
                        if let Arg::Int(amount) = rule.args[1] {
                            amount
                        } else {
                            0
                        },
                    ),
                    "KilledNpcs" => (
                        QuestRequirementIcon::Kill,
                        if let Arg::Int(npc_id) = rule.args[0] {
                            q.get_npc_kills(npc_id)
                        } else {
                            0
                        },
                        if let Arg::Int(amount) = rule.args[1] {
                            amount
                        } else {
                            0
                        },
                    ),
                    "KilledPlayers" => (
                        QuestRequirementIcon::Kill,
                        q.player_kills,
                        if let Arg::Int(amount) = rule.args[0] {
                            amount
                        } else {
                            0
                        },
                    ),
                    "EnterCoord" | "EnterMap" | "LeaveMap" => (QuestRequirementIcon::Step, 0, 0),
                    _ => (QuestRequirementIcon::Talk, 0, 0),
                };

                Some(QuestProgressEntry {
                    name: quest.name.to_owned(),
                    description: state.description.to_owned(),
                    icon,
                    progress,
                    target,
                })
            })
            .collect::<Vec<_>>();

        player.send(
            PacketAction::List,
            PacketFamily::Quest,
            &QuestListServerPacket {
                page: QuestPage::Progress,
                quests_count: quest_progress_entries.len() as i32,
                page_data: Some(QuestListServerPacketPageData::Progress(
                    QuestListServerPacketPageDataProgress {
                        quest_progress_entries,
                    },
                )),
            },
        )
    }
}
