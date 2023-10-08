use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::statskill::{Reply, ReplyData, ReplyWrongClass, Take},
        PacketAction, PacketFamily, SkillMasterReply,
    },
    pubs::EnfNpcType,
};

use crate::{NPC_DB, SKILL_MASTER_DB};

use super::Map;

impl Map {
    pub async fn learn_skill(
        &mut self,
        player_id: EOShort,
        spell_id: EOShort,
        session_id: EOShort,
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

        if npc_data.r#type != EnfNpcType::Skills {
            return;
        }

        let skill_master = match SKILL_MASTER_DB
            .skill_masters
            .iter()
            .find(|skill_master| skill_master.vendor_id == npc_data.behavior_id)
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
            || character.adj_strength < skill.str_req
            || character.adj_intelligence < skill.int_req
            || character.adj_wisdom < skill.wis_req
            || character.adj_agility < skill.agi_req
            || character.adj_constitution < skill.con_req
            || character.adj_charisma < skill.cha_req
            || (skill.skill_id_req1 > 0 && !character.has_spell(skill.skill_id_req1))
            || (skill.skill_id_req2 > 0 && !character.has_spell(skill.skill_id_req2))
            || (skill.skill_id_req3 > 0 && !character.has_spell(skill.skill_id_req3))
            || (skill.skill_id_req4 > 0 && !character.has_spell(skill.skill_id_req4))
        {
            return;
        }

        if skill.class_req > 0 && character.class != skill.class_req {
            let reply = Reply {
                reply_code: SkillMasterReply::WrongClass,
                data: ReplyData::WrongClass(ReplyWrongClass {
                    class_id: character.class as EOShort,
                }),
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            character.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::StatSkill,
                builder.get(),
            );

            return;
        }

        character.remove_item(1, skill.price);
        character.add_spell(skill.skill_id);

        let reply = Take {
            spell_id,
            gold_amount: character.get_item_amount(1),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Take,
            PacketFamily::StatSkill,
            builder.get(),
        );
    }
}
