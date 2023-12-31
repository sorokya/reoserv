use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{
                RecoverPlayerServerPacket, SpellTargetOtherServerPacket,
                SpellTargetSelfServerPacket,
            },
            PacketAction, PacketFamily,
        },
        r#pub::{EsfRecord, NpcType, SkillTargetRestrict, SkillTargetType, SkillType},
    },
};
use rand::Rng;

use crate::{
    character::{SpellState, SpellTarget},
    NPC_DB, SPELL_DB,
};

use super::super::Map;

impl Map {
    pub async fn cast_spell(&mut self, player_id: i32, target: SpellTarget) {
        let spell_id = match self.get_player_spell_id(player_id) {
            Some(spell_id) => spell_id,
            None => return,
        };

        let spell_data = match SPELL_DB.skills.get(spell_id as usize - 1) {
            Some(spell_data) => spell_data,
            None => return,
        };

        match spell_data.r#type {
            SkillType::Heal => self.cast_heal_spell(player_id, spell_id, spell_data, target),
            SkillType::Attack => {
                self.cast_damage_spell(player_id, spell_id, spell_data, target)
                    .await
            }
            SkillType::Bard => {}
            _ => {}
        }
    }

    fn get_player_spell_id(&self, player_id: i32) -> Option<i32> {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return None,
        };

        match character.spell_state {
            SpellState::Requested {
                spell_id,
                timestamp: _,
                cast_time: _,
            } => {
                // TODO: enforce timestamp
                if character.has_spell(spell_id) {
                    Some(spell_id)
                } else {
                    None
                }
            }
            SpellState::None => None,
        }
    }

    fn cast_heal_spell(
        &mut self,
        player_id: i32,
        spell_id: i32,
        spell: &EsfRecord,
        target: SpellTarget,
    ) {
        if spell.target_restrict != SkillTargetRestrict::Friendly {
            return;
        }

        match target {
            SpellTarget::Player => self.cast_heal_self(player_id, spell_id, spell),
            SpellTarget::Group => self.cast_heal_group(player_id, spell),
            SpellTarget::OtherPlayer(target_player_id) => {
                self.cast_heal_player(player_id, target_player_id, spell_id, spell)
            }
            _ => {}
        }
    }

    fn cast_heal_self(&mut self, player_id: i32, spell_id: i32, spell: &EsfRecord) {
        if spell.target_type != SkillTargetType::SELF {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell.tp_cost {
            return;
        }

        character.spell_state = SpellState::None;
        character.tp -= spell.tp_cost;
        let original_hp = character.hp;
        character.hp = cmp::min(character.hp + spell.hp_heal, character.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let hp_percentage = character.get_hp_percentage();

        if character.hp != original_hp {
            character
                .player
                .as_ref()
                .unwrap()
                .update_party_hp(hp_percentage);
        }

        let packet = SpellTargetSelfServerPacket {
            player_id,
            spell_id,
            spell_heal_hp: spell.hp_heal,
            hp_percentage,
            hp: None,
            tp: None,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize SpellTargetSelfServerPacket: {}", e);
            return;
        }

        self.send_buf_near_player(
            player_id,
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            writer.to_byte_array(),
        );

        let packet = SpellTargetSelfServerPacket {
            player_id,
            spell_id,
            spell_heal_hp: spell.hp_heal,
            hp_percentage,
            hp: Some(character.hp),
            tp: Some(character.tp),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize SpellTargetSelfServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            writer.to_byte_array(),
        );
    }

    fn cast_heal_group(&mut self, _player_id: i32, _spell: &EsfRecord) {}

    fn cast_heal_player(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        spell_id: i32,
        spell: &EsfRecord,
    ) {
        if spell.target_type != SkillTargetType::Normal {
            return;
        }

        if !self.characters.contains_key(&target_player_id) {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell.tp_cost {
            return;
        }

        character.spell_state = SpellState::None;
        character.tp -= spell.tp_cost;

        let target = match self.characters.get_mut(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let original_hp = target.hp;
        target.hp = cmp::min(target.hp + spell.hp_heal, target.max_hp);
        let hp_percentage = target.get_hp_percentage();

        if target.hp != original_hp {
            target
                .player
                .as_ref()
                .unwrap()
                .update_party_hp(hp_percentage);
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let mut packet: SpellTargetOtherServerPacket = SpellTargetOtherServerPacket {
            victim_id: target_player_id,
            caster_id: player_id,
            caster_direction: character.direction,
            spell_id,
            spell_heal_hp: spell.hp_heal,
            hp_percentage: target.get_hp_percentage(),
            hp: None,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize SpellTargetOtherServerPacket: {}", e);
            return;
        }

        self.send_buf_near_player(
            target_player_id,
            PacketAction::TargetOther,
            PacketFamily::Spell,
            writer.to_byte_array(),
        );

        packet.hp = Some(target.hp);

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize SpellTargetOtherServerPacket: {}", e);
            return;
        }

        target.player.as_ref().unwrap().send(
            PacketAction::TargetOther,
            PacketFamily::Spell,
            writer.to_byte_array(),
        );

        let packet = RecoverPlayerServerPacket {
            hp: character.hp,
            tp: character.tp,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize RecoverPlayerServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::Recover,
            writer.to_byte_array(),
        );
    }

    async fn cast_damage_spell(
        &mut self,
        player_id: i32,
        spell_id: i32,
        spell_data: &EsfRecord,
        target: SpellTarget,
    ) {
        if spell_data.target_restrict == SkillTargetRestrict::Friendly
            || spell_data.target_type != SkillTargetType::Normal
        {
            return;
        }

        match target {
            SpellTarget::Npc(npc_index) => {
                self.cast_damage_npc(player_id, npc_index, spell_id, spell_data)
                    .await
            }
            SpellTarget::OtherPlayer(_) => warn!("Spell PVP not implemented yet"),
            _ => {}
        }
    }

    async fn cast_damage_npc(
        &mut self,
        player_id: i32,
        npc_index: i32,
        spell_id: i32,
        spell_data: &EsfRecord,
    ) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell_data.tp_cost {
            return;
        }

        let direction = character.direction;

        let npc = match self.npcs.get_mut(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if !matches!(npc_data.r#type, NpcType::Passive | NpcType::Aggressive) {
            return;
        }

        let amount = {
            let mut rng = rand::thread_rng();
            rng.gen_range(
                character.min_damage + spell_data.min_damage
                    ..=character.max_damage + spell_data.max_damage,
            )
        };

        let critical = npc.hp == npc.max_hp;

        let damage_dealt = npc.damage(player_id, amount, character.accuracy, critical);

        if npc.alive {
            self.attack_npc_reply(
                player_id,
                npc_index,
                direction,
                damage_dealt,
                Some(spell_id),
            );
        } else {
            self.attack_npc_killed_reply(
                player_id,
                npc_index,
                direction,
                damage_dealt,
                Some(spell_id),
            )
            .await;
        }
    }
}
