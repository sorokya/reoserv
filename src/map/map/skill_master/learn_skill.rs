use eolib::{protocol::{r#pub::NpcType, net::{server::{StatSkillReplyServerPacket, SkillMasterReply, StatSkillReplyServerPacketReplyCodeData, StatSkillReplyServerPacketReplyCodeDataWrongClass, StatSkillTakeServerPacket}, PacketAction, PacketFamily}}, data::{EoWriter, EoSerialize}};

use crate::{NPC_DB, SKILL_MASTER_DB};

use super::super::Map;

impl Map {
    pub async fn learn_skill(
        &mut self,
        player_id: i32,
        spell_id: i32,
        session_id: i32,
    ) {
        if spell_id == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let actual_session_id = match character.player.as_ref().unwrap().get_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get session id {}", e);
                return;
            }
        };

        if actual_session_id != session_id {
            return;
        }

        let npc_index = match character
            .player
            .as_ref()
            .unwrap()
            .get_interact_npc_index()
            .await
        {
            Some(index) => index,
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
            || (skill.skill_id_requirement1 > 0 && !character.has_spell(skill.skill_id_requirement1))
            || (skill.skill_id_requirement2 > 0 && !character.has_spell(skill.skill_id_requirement2))
            || (skill.skill_id_requirement3 > 0 && !character.has_spell(skill.skill_id_requirement3))
            || (skill.skill_id_requirement4 > 0 && !character.has_spell(skill.skill_id_requirement4))
        {
            return;
        }

        if skill.class_requirement > 0 && character.class != skill.class_requirement {
            let reply = StatSkillReplyServerPacket {
                reply_code: SkillMasterReply::WrongClass,
                reply_code_data: Some(StatSkillReplyServerPacketReplyCodeData::WrongClass(StatSkillReplyServerPacketReplyCodeDataWrongClass {
                    class_id: character.class,
                })),
            };

            let mut writer = EoWriter::new();
            reply.serialize(&mut writer);

            character.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::StatSkill,
                writer.to_byte_array(),
            );

            return;
        }

        character.remove_item(1, skill.price);
        character.add_spell(skill.skill_id);

        let reply = StatSkillTakeServerPacket {
            spell_id,
            gold_amount: character.get_item_amount(1),
        };

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);

        character.player.as_ref().unwrap().send(
            PacketAction::Take,
            PacketFamily::StatSkill,
            writer.to_byte_array(),
        );
    }
}
