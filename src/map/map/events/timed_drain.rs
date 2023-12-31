use std::cmp;

use eolib::{
    data::EoWriter,
    protocol::{
        map::MapTimedEffect,
        net::{PacketAction, PacketFamily},
    },
};

use crate::{utils::in_client_range, SETTINGS};

use super::super::Map;

const EFFECT_DRAIN: i32 = 1;

impl Map {
    pub fn timed_drain(&mut self) {
        if self.file.timed_effect == MapTimedEffect::HpDrain {
            self.timed_drain_hp();
        }

        if self.file.timed_effect == MapTimedEffect::TpDrain {
            self.timed_drain_tp();
        }
    }

    fn timed_drain_hp(&mut self) {
        let player_ids: Vec<i32> = self.characters.keys().copied().collect();
        let mut damage_list: Vec<i32> = Vec::with_capacity(player_ids.len());

        for player_id in &player_ids {
            let character = match self.characters.get_mut(player_id) {
                Some(character) => character,
                None => {
                    damage_list.push(0);
                    continue;
                }
            };

            let damage = (character.max_hp as f32 * SETTINGS.world.drain_hp_damage).floor() as i32;
            let damage = cmp::min(damage, character.hp - 1);
            let damage = cmp::max(damage, 0) as i32;

            character.hp -= damage;
            damage_list.push(damage);
        }

        for (index, player_id) in player_ids.iter().enumerate() {
            let damage = match damage_list.get(index) {
                Some(damage) => *damage,
                None => 0,
            };

            let character = match self.characters.get(player_id) {
                Some(character) => character,
                None => continue,
            };

            let mut writer = EoWriter::new();
            writer.add_short(damage);
            writer.add_short(character.hp);
            writer.add_short(character.max_hp);

            for (other_index, other_player_id) in player_ids.iter().enumerate() {
                if other_player_id == player_id {
                    continue;
                }

                let other = match self.characters.get(other_player_id) {
                    Some(other) => other,
                    None => continue,
                };

                if other.hidden || !in_client_range(&character.coords, &other.coords) {
                    continue;
                }

                let other_damage = match damage_list.get(other_index) {
                    Some(damage) => *damage,
                    None => 0,
                };

                writer.add_short(*other_player_id);
                writer.add_char(other.get_hp_percentage());
                writer.add_short(other_damage);
            }

            character.player.as_ref().unwrap().send(
                PacketAction::TargetOther,
                PacketFamily::Effect,
                writer.to_byte_array(),
            );
        }
    }

    fn timed_drain_tp(&mut self) {
        for character in self.characters.values_mut() {
            if character.tp == 0 {
                continue;
            }

            let damage = (character.max_tp as f32 * SETTINGS.world.drain_tp_damage).floor() as i32;
            let damage = cmp::min(damage, character.tp - 1);
            let damage = cmp::max(damage, 0) as i32;

            character.tp -= damage;

            let mut writer = EoWriter::new();
            writer.add_char(EFFECT_DRAIN);
            writer.add_short(damage);
            writer.add_short(character.tp);
            writer.add_short(character.max_tp);

            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Effect,
                writer.to_byte_array(),
            );
        }
    }
}
