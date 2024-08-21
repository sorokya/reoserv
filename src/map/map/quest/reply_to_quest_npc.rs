use std::cmp::Ordering;

use eolib::protocol::{
    net::{
        server::{
            DialogEntry, DialogEntryEntryTypeData, DialogEntryEntryTypeDataLink, DialogEntryType,
            DialogQuestEntry, QuestDialogServerPacket, QuestReportServerPacket,
        },
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};
use eoplus::{Arg, Quest};

use crate::{NPC_DB, QUEST_DB};

use super::super::Map;

impl Map {
    pub fn reply_to_quest_npc(
        &mut self,
        player_id: i32,
        npc_index: i32,
        quest_id: i32,
        session_id: i32,
        action_id: Option<i32>,
    ) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Quest {
            return;
        }

        let previous_state = character.get_quest_progress(quest_id).state;

        character.talked_to_npc(npc_data.behavior_id, quest_id, action_id);

        let quests_for_npc = QUEST_DB
            .iter()
            .filter(|(id, quest)| {
                let progress = character.get_quest_progress(**id);
                quest.states[progress.state as usize]
                    .actions
                    .iter()
                    .any(|action| {
                        (action.name == "AddNpcText" || action.name == "AddNpcInput")
                            && action.args[0] == Arg::Int(npc_data.behavior_id)
                    })
            })
            .collect::<Vec<(&i32, &Quest)>>();

        if quests_for_npc.is_empty() {
            return;
        }

        let (quest_id, quest) = if quest_id > 0 {
            match quests_for_npc.iter().find(|(id, _)| **id == quest_id) {
                Some((id, quest)) => (**id, quest),
                None => return,
            }
        } else {
            (*quests_for_npc[0].0, &quests_for_npc[0].1)
        };

        let progress = character.get_quest_progress(quest_id);

        // Only show new dialog the first time quest state advances
        if previous_state >= progress.state {
            return;
        }

        let dialog_entries = quest.states[progress.state as usize]
            .actions
            .iter()
            .filter_map(|action| {
                if (action.name == "AddNpcText" || action.name == "AddNpcInput")
                    && action.args[0] == Arg::Int(npc_data.behavior_id)
                {
                    Some(DialogEntry {
                        entry_type: if action.name == "AddNpcText" {
                            DialogEntryType::Text
                        } else {
                            DialogEntryType::Link
                        },
                        entry_type_data: if action.name == "AddNpcText" {
                            None
                        } else {
                            Some(DialogEntryEntryTypeData::Link(
                                DialogEntryEntryTypeDataLink {
                                    link_id: match action.args[1] {
                                        Arg::Int(link_id) => link_id,
                                        _ => {
                                            error!("Invalid argument for AddNpcText");
                                            return None;
                                        }
                                    },
                                },
                            ))
                        },
                        line: match &action.args[if action.name == "AddNpcText" { 1 } else { 2 }] {
                            Arg::Str(line) => line.to_owned(),
                            _ => {
                                error!("Could not find NPC dialog line");
                                return None;
                            }
                        },
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<DialogEntry>>();

        let mut quest_entries = quests_for_npc
            .iter()
            .filter_map(|(quest_id, quest)| {
                let progress = character.get_quest_progress(**quest_id);

                if quest.states[progress.state as usize]
                    .actions
                    .iter()
                    .any(|action| {
                        (action.name == "AddNpcText" || action.name == "AddNpcInput")
                            && action.args[0] == Arg::Int(npc_data.behavior_id)
                    })
                {
                    Some(DialogQuestEntry {
                        quest_id: **quest_id,
                        quest_name: quest.name.to_owned(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<DialogQuestEntry>>();

        quest_entries.sort_by(|a, b| {
            if a.quest_id == quest_id {
                Ordering::Less
            } else {
                a.quest_id.partial_cmp(&b.quest_id).unwrap()
            }
        });

        character.save_quest_progress(quest_id, progress.state);

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        player.send(
            PacketAction::Dialog,
            PacketFamily::Quest,
            &QuestDialogServerPacket {
                behavior_id: npc_data.behavior_id,
                quest_id,
                session_id,
                dialog_id: 0,
                quest_entries,
                dialog_entries,
            },
        );

        let messages = quest.states[progress.state as usize]
            .actions
            .iter()
            .filter_map(|action| {
                if action.name == "AddNpcChat" && action.args[0] == Arg::Int(npc_data.behavior_id) {
                    Some(match &action.args[1] {
                        Arg::Str(message) => message.to_owned(),
                        _ => {
                            error!("Invalid argument for AddNpcChat");
                            return None;
                        }
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        if !messages.is_empty() {
            let packet = QuestReportServerPacket {
                npc_index,
                messages,
            };

            self.send_packet_near(
                &npc.coords,
                PacketAction::Report,
                PacketFamily::Quest,
                packet,
            );
        }
    }
}
