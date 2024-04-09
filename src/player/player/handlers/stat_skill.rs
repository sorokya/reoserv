use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            StatSkillAddClientPacket, StatSkillAddClientPacketActionTypeData,
            StatSkillJunkClientPacket, StatSkillOpenClientPacket, StatSkillRemoveClientPacket,
            StatSkillTakeClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn stat_skill_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match StatSkillAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing StatSkillAddClientPacket {}", e);
                    return;
                }
            };

            match add.action_type_data {
                Some(StatSkillAddClientPacketActionTypeData::Stat(stat)) => {
                    map.level_stat(self.id, stat.stat_id)
                }
                Some(StatSkillAddClientPacketActionTypeData::Skill(skill)) => {
                    map.level_skill(self.id, skill.spell_id)
                }
                _ => {}
            }
        }
    }

    fn stat_skill_junk(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let junk = match StatSkillJunkClientPacket::deserialize(&reader) {
                Ok(junk) => junk,
                Err(e) => {
                    error!("Error deserializing StatSkillJunkClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != junk.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.reset_character(self.id, npc_index);
        }
    }

    fn stat_skill_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match StatSkillOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing StatSkillOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_skill_master(self.id, open.npc_index, session_id);
        }
    }

    fn stat_skill_remove(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let remove = match StatSkillRemoveClientPacket::deserialize(&reader) {
                Ok(remove) => remove,
                Err(e) => {
                    error!("Error deserializing StatSkillRemoveClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != remove.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.forget_skill(self.id, npc_index, remove.spell_id);
        }
    }

    fn stat_skill_take(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let take = match StatSkillTakeClientPacket::deserialize(&reader) {
                Ok(take) => take,
                Err(e) => {
                    error!("Error deserializing StatSkillTakeClientPacket {}", e);
                    return;
                }
            };

            // Prevent learning new skills while trading
            if self.trading {
                return;
            }

            match self.session_id {
                Some(session_id) => {
                    if session_id != take.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.learn_skill(self.id, npc_index, take.spell_id);
        }
    }

    pub fn handle_stat_skill(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Add => self.stat_skill_add(reader),
            PacketAction::Junk => self.stat_skill_junk(reader),
            PacketAction::Open => self.stat_skill_open(reader),
            PacketAction::Remove => self.stat_skill_remove(reader),
            PacketAction::Take => self.stat_skill_take(reader),
            _ => error!("Unhandled packet StatSkill_{:?}", action),
        }
    }
}
