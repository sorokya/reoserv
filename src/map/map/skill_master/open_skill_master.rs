use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::statskill::Open, CharacterBaseStats, PacketAction, PacketFamily, SkillLearn,
    },
    pubs::EnfNpcType,
};

use crate::{NPC_DB, SKILL_MASTER_DB};

use super::super::Map;

impl Map {
    pub async fn open_skill_master(&mut self, player_id: EOShort, npc_index: EOChar) {
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

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let session_id = match player.generate_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to generate session id {}", e);
                return;
            }
        };

        player.set_interact_npc_index(npc_index);

        let reply = Open {
            session_id,
            shop_name: skill_master.name.clone(),
            skills: skill_master
                .skills
                .iter()
                .map(|skill| SkillLearn {
                    id: skill.skill_id,
                    level_req: skill.min_level,
                    class_req: skill.class_req,
                    cost: skill.price,
                    skill_req: [
                        skill.skill_id_req1,
                        skill.skill_id_req2,
                        skill.skill_id_req3,
                        skill.skill_id_req4,
                    ],
                    stat_req: CharacterBaseStats {
                        str: skill.str_req,
                        intl: skill.int_req,
                        wis: skill.wis_req,
                        agi: skill.agi_req,
                        con: skill.con_req,
                        cha: skill.cha_req,
                    },
                })
                .collect(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Open, PacketFamily::StatSkill, builder.get());
    }
}
