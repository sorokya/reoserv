use eolib::protocol::{
    net::{
        server::{
            SkillMasterReply, StatSkillReplyServerPacket, StatSkillReplyServerPacketReplyCodeData,
            StatSkillReplyServerPacketReplyCodeDataWrongClass, StatSkillTakeServerPacket,
        },
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{NPC_DB, SKILL_MASTER_DB};

use super::super::Map;

impl Map {
    pub fn learn_skill(&mut self, player_id: i32, npc_index: i32, spell_id: i32) {
        if spell_id <= 0 {
            return;
        }

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

        if npc_data.r#type != NpcType::Trainer {
            return;
        }

        let skill_master = match SKILL_MASTER_DB
            .skill_masters
            .iter()
            .find(|skill_master| skill_master.behavior_id == npc_data.behavior_id)
        {
            Some(skill_master) => skill_master,
            None => return,
        };

        let skill = match skill_master
            .skills
            .iter()
            .find(|skill| skill.skill_id == spell_id)
        {
            Some(skill) => skill,
            None => return,
        };

        if character.get_item_amount(1) < skill.price
            || character.adj_strength < skill.str_requirement
            || character.adj_intelligence < skill.int_requirement
            || character.adj_wisdom < skill.wis_requirement
            || character.adj_agility < skill.agi_requirement
            || character.adj_constitution < skill.con_requirement
            || character.adj_charisma < skill.cha_requirement
            || skill
                .skill_requirements
                .iter()
                .any(|s| *s > 0 && !character.has_spell(*s))
        {
            return;
        }

        if skill.class_requirement > 0 && character.class != skill.class_requirement {
            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::StatSkill,
                    &StatSkillReplyServerPacket {
                        reply_code: SkillMasterReply::WrongClass,
                        reply_code_data: Some(StatSkillReplyServerPacketReplyCodeData::WrongClass(
                            StatSkillReplyServerPacketReplyCodeDataWrongClass {
                                class_id: skill.class_requirement,
                            },
                        )),
                    },
                );
            }

            return;
        }

        character.remove_item(1, skill.price);
        character.add_spell(skill.skill_id);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Take,
                PacketFamily::StatSkill,
                &StatSkillTakeServerPacket {
                    spell_id,
                    gold_amount: character.get_item_amount(1),
                },
            );
        }
    }
}
