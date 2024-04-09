use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{SpellRequestClientPacket, SpellTargetOtherClientPacket, SpellTargetType},
        PacketAction,
    },
};

use crate::character::SpellTarget;

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

            map.start_spell_chant(self.id, request.spell_id, request.timestamp);
        }
    }

    fn spell_target_self(&mut self) {
        if let Some(map) = &self.map {
            map.cast_spell(self.id, SpellTarget::Player);
        }
    }

    fn spell_target_other(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let target_other = match SpellTargetOtherClientPacket::deserialize(&reader) {
                Ok(target_other) => target_other,
                Err(e) => {
                    error!("Error deserializing SpellTargetOtherClientPacket {}", e);
                    return;
                }
            };

            match target_other.target_type {
                SpellTargetType::Player => {
                    map.cast_spell(self.id, SpellTarget::OtherPlayer(target_other.victim_id))
                }
                SpellTargetType::Npc => {
                    map.cast_spell(self.id, SpellTarget::Npc(target_other.victim_id))
                }
                _ => {}
            }
        }
    }

    fn spell_target_group(&mut self) {
        if let Some(map) = &self.map {
            map.cast_spell(self.id, SpellTarget::Group);
        }
    }

    pub fn handle_spell(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.spell_request(reader),
            PacketAction::TargetSelf => self.spell_target_self(),
            PacketAction::TargetOther => self.spell_target_other(reader),
            PacketAction::TargetGroup => self.spell_target_group(),
            _ => error!("Unhandled packet Spell_{:?}", action),
        }
    }
}
