use std::cmp;

use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            SpellRequestClientPacket, SpellTargetGroupClientPacket, SpellTargetOtherClientPacket,
            SpellTargetSelfClientPacket, SpellTargetType,
        },
        PacketAction,
    },
};

use crate::{character::SpellTarget, utils::timestamp_diff, SPELL_DB};

use super::super::Player;

impl Player {
    fn spell_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match SpellRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing SpellRequestClientPacket {}", e);
                    return;
                }
            };

            if request.spell_id <= 0 {
                return;
            }

            self.timestamp = request.timestamp;
            self.spell_id = Some(request.spell_id);

            map.start_spell_chant(self.id, request.spell_id);
        }
    }

    fn spell_target_self(&mut self, reader: EoReader) {
        let target_self = match SpellTargetSelfClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing SpellTargetSelfClientPacket {}", e);
                return;
            }
        };

        match self.spell_id {
            Some(spell_id) => {
                if spell_id != target_self.spell_id {
                    return;
                }
            }
            None => return,
        }

        if !self.check_timestamp(target_self.spell_id, target_self.timestamp) {
            return;
        }

        self.timestamp = target_self.timestamp;
        self.spell_id = None;

        if let Some(map) = &self.map {
            map.cast_spell(self.id, target_self.spell_id, SpellTarget::Player);
        }
    }

    fn spell_target_other(&mut self, reader: EoReader) {
        let target_other = match SpellTargetOtherClientPacket::deserialize(&reader) {
            Ok(target_other) => target_other,
            Err(e) => {
                error!("Error deserializing SpellTargetOtherClientPacket {}", e);
                return;
            }
        };

        match self.spell_id {
            Some(spell_id) => {
                if spell_id != target_other.spell_id {
                    return;
                }
            }
            None => return,
        }

        if !self.check_timestamp(target_other.spell_id, target_other.timestamp) {
            return;
        }

        self.timestamp = target_other.timestamp;
        self.spell_id = None;

        if let Some(map) = &self.map {
            match target_other.target_type {
                SpellTargetType::Player => map.cast_spell(
                    self.id,
                    target_other.spell_id,
                    SpellTarget::OtherPlayer(target_other.victim_id),
                ),
                SpellTargetType::Npc => map.cast_spell(
                    self.id,
                    target_other.spell_id,
                    SpellTarget::Npc(target_other.victim_id),
                ),
                _ => {}
            }
        }
    }

    fn spell_target_group(&mut self, reader: EoReader) {
        let target_group = match SpellTargetGroupClientPacket::deserialize(&reader) {
            Ok(target_other) => target_other,
            Err(e) => {
                error!("Error deserializing SpellTargetGroupClientPacket {}", e);
                return;
            }
        };

        match self.spell_id {
            Some(spell_id) => {
                if spell_id != target_group.spell_id {
                    return;
                }
            }
            None => return,
        }

        if !self.check_timestamp(target_group.spell_id, target_group.timestamp) {
            return;
        }

        self.timestamp = target_group.timestamp;
        self.spell_id = None;

        if let Some(map) = &self.map {
            map.cast_spell(self.id, target_group.spell_id, SpellTarget::Group);
        }
    }

    fn check_timestamp(&mut self, spell_id: i32, timestamp: i32) -> bool {
        let spell = match SPELL_DB.skills.get(spell_id as usize - 1) {
            Some(spell) => spell,
            None => return false,
        };

        let diff = timestamp_diff(timestamp, self.timestamp);

        diff >= (spell.cast_time - 1) * 47 + 35 && diff < cmp::max(spell.cast_time, 1) * 50
    }

    pub fn handle_spell(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.spell_request(reader),
            PacketAction::TargetSelf => self.spell_target_self(reader),
            PacketAction::TargetOther => self.spell_target_other(reader),
            PacketAction::TargetGroup => self.spell_target_group(reader),
            _ => error!("Unhandled packet Spell_{:?}", action),
        }
    }
}
