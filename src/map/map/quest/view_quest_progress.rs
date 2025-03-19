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

                if let Some(rule) = state.rules.iter().find(|rule| rule.name == "GotItems") {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Item,
                        progress: if let Arg::Int(item_id) = rule.args[0] {
                            character.get_item_amount(item_id)
                        } else {
                            0
                        },
                        target: if let Arg::Int(amount) = rule.args[1] {
                            amount
                        } else {
                            0
                        },
                    });
                }

                if state.rules.iter().any(|rule| rule.name == "EquippedItem") {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Item,
                        progress: 0,
                        target: 1,
                    });
                }

                if state.rules.iter().any(|rule| rule.name == "UnequippedItem") {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Item,
                        progress: 0,
                        target: 1,
                    });
                }

                if let Some(rule) = state.rules.iter().find(|rule| rule.name == "KilledNpcs") {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Kill,
                        progress: if let Arg::Int(npc_id) = rule.args[0] {
                            q.get_npc_kills(npc_id)
                        } else {
                            0
                        },
                        target: if let Arg::Int(amount) = rule.args[1] {
                            amount
                        } else {
                            0
                        },
                    });
                }

                if let Some(rule) = state.rules.iter().find(|rule| rule.name == "KilledPlayers") {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Kill,
                        progress: q.player_kills,
                        target: if let Arg::Int(amount) = rule.args[1] {
                            amount
                        } else {
                            0
                        },
                    });
                }

                if state.rules.iter().any(|rule| {
                    rule.name == "EnterCoord" || rule.name == "EnterMap" || rule.name == "LeaveMap"
                }) {
                    return Some(QuestProgressEntry {
                        name: quest.name.to_owned(),
                        description: state.description.to_owned(),
                        icon: QuestRequirementIcon::Step,
                        progress: 0,
                        target: 0,
                    });
                }

                Some(QuestProgressEntry {
                    name: quest.name.to_owned(),
                    description: state.description.to_owned(),
                    icon: QuestRequirementIcon::Talk,
                    progress: 0,
                    target: 0,
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
