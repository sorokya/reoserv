use std::cmp;

use eo::{
    data::{EOChar, EOInt, EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
    pubs::{EnfNpcType, EsfSpell, EsfSpellTargetRestrict, EsfSpellTargetType, EsfSpellType},
};
use rand::Rng;

use crate::{
    character::{SpellState, SpellTarget},
    NPC_DB, SPELL_DB,
};

use super::Map;

impl Map {
    pub fn cast_spell(&mut self, player_id: EOShort, target: SpellTarget) {
        let spell_id = match self.get_player_spell_id(player_id) {
            Some(spell_id) => spell_id,
            None => return,
        };

        let spell_data = match SPELL_DB.spells.get(spell_id as usize - 1) {
            Some(spell_data) => spell_data,
            None => return,
        };

        match spell_data.r#type {
            EsfSpellType::Heal => self.cast_heal_spell(player_id, spell_id, spell_data, target),
            EsfSpellType::Damage => self.cast_damage_spell(player_id, spell_id, spell_data, target),
            EsfSpellType::Bard => {}
        }
    }

    fn get_player_spell_id(&self, player_id: EOShort) -> Option<EOShort> {
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
        player_id: EOShort,
        spell_id: EOShort,
        spell: &EsfSpell,
        target: SpellTarget,
    ) {
        if spell.target_restrict != EsfSpellTargetRestrict::Friendly {
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

    fn cast_heal_self(&mut self, player_id: EOShort, spell_id: EOShort, spell: &EsfSpell) {
        if spell.target_type != EsfSpellTargetType::Player {
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
        character.hp = cmp::min(character.hp + spell.hp_heal, character.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_short(spell_id);
        builder.add_int(spell.hp_heal as EOInt);
        builder.add_char(character.get_hp_percentage());

        self.send_buf_near_player(
            player_id,
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            builder.get(),
        );

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_short(spell_id);
        builder.add_int(spell.hp_heal as EOInt);
        builder.add_char(character.get_hp_percentage());
        builder.add_short(character.hp);
        builder.add_short(character.tp);

        character.player.as_ref().unwrap().send(
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            builder.get(),
        );
    }

    fn cast_heal_group(&mut self, _player_id: EOShort, _spell: &EsfSpell) {}

    fn cast_heal_player(
        &mut self,
        player_id: EOShort,
        target_player_id: EOShort,
        spell_id: EOShort,
        spell: &EsfSpell,
    ) {
        if spell.target_type != EsfSpellTargetType::Other {
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

        target.hp = cmp::min(target.hp + spell.hp_heal, target.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let mut builder = StreamBuilder::new();
        builder.add_short(target_player_id);
        builder.add_short(player_id);
        builder.add_char(character.direction.to_char());
        builder.add_short(spell_id);
        builder.add_int(spell.hp_heal as EOInt);
        builder.add_char(target.get_hp_percentage());

        self.send_buf_near_player(
            target_player_id,
            PacketAction::TargetOther,
            PacketFamily::Spell,
            builder.get(),
        );

        let mut builder = StreamBuilder::new();
        builder.add_short(target_player_id);
        builder.add_short(player_id);
        builder.add_char(character.direction.to_char());
        builder.add_short(spell_id);
        builder.add_int(spell.hp_heal as EOInt);
        builder.add_char(target.get_hp_percentage());
        builder.add_short(target.hp);

        target.player.as_ref().unwrap().send(
            PacketAction::TargetOther,
            PacketFamily::Spell,
            builder.get(),
        );

        let mut builder = StreamBuilder::new();
        builder.add_short(character.hp);
        builder.add_short(character.tp);

        character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::Recover,
            builder.get(),
        );
    }

    fn cast_damage_spell(
        &mut self,
        player_id: EOShort,
        spell_id: EOShort,
        spell_data: &EsfSpell,
        target: SpellTarget,
    ) {
        if spell_data.target_restrict == EsfSpellTargetRestrict::Friendly
            || spell_data.target_type != EsfSpellTargetType::Other
        {
            return;
        }

        match target {
            SpellTarget::Npc(npc_index) => {
                self.cast_damage_npc(player_id, npc_index, spell_id, spell_data)
            }
            SpellTarget::OtherPlayer(_) => warn!("Spell PVP not implemented yet"),
            _ => {}
        }
    }

    fn cast_damage_npc(
        &mut self,
        player_id: EOShort,
        npc_index: EOChar,
        spell_id: EOShort,
        spell_data: &EsfSpell,
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

        if !matches!(
            npc_data.r#type,
            EnfNpcType::Passive | EnfNpcType::Aggressive
        ) {
            return;
        }

        let mut rng = rand::thread_rng();

        let amount = rng.gen_range(
            character.min_damage + spell_data.min_damage
                ..=character.max_damage + spell_data.max_damage,
        );

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
            );
        }
    }
}
