use eolib::protocol::{
    net::{
        server::{CharacterBaseStats, SkillLearn, StatSkillOpenServerPacket},
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{NPC_DB, SKILL_MASTER_DB};

use super::super::Map;

impl Map {
    pub async fn open_skill_master(&mut self, player_id: i32, npc_index: i32) {
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

        player.send(
            PacketAction::Open,
            PacketFamily::StatSkill,
            &StatSkillOpenServerPacket {
                session_id,
                shop_name: skill_master.name.clone(),
                skills: skill_master
                    .skills
                    .iter()
                    .map(|skill| SkillLearn {
                        id: skill.skill_id,
                        level_requirement: skill.level_requirement,
                        class_requirement: skill.class_requirement,
                        cost: skill.price,
                        skill_requirements: skill.skill_requirements,
                        stat_requirements: CharacterBaseStats {
                            str: skill.str_requirement,
                            intl: skill.int_requirement,
                            wis: skill.wis_requirement,
                            agi: skill.agi_requirement,
                            con: skill.con_requirement,
                            cha: skill.cha_requirement,
                        },
                    })
                    .collect(),
            },
        );
    }
}
