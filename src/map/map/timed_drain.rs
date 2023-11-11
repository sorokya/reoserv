use std::cmp;

use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
    pubs::EmfEffect,
};

use crate::{utils::in_client_range, SETTINGS};

use super::Map;

impl Map {
    pub fn timed_drain(&mut self) {
        if self.file.effect == EmfEffect::HPDrain {
            self.timed_drain_hp();
        }

        if self.file.effect == EmfEffect::TPDrain {
            self.timed_drain_tp();
        }
    }

    fn timed_drain_hp(&mut self) {
        let player_ids: Vec<EOShort> = self.characters.keys().copied().collect();
        let mut damage_list: Vec<EOShort> = Vec::with_capacity(player_ids.len());

        for player_id in &player_ids {
            let character = match self.characters.get_mut(player_id) {
                Some(character) => character,
                None => {
                    damage_list.push(0);
                    continue;
                }
            };

            let damage = (character.max_hp as f32 * SETTINGS.map.drain_hp_damage).floor() as i32;
            let damage = cmp::min(damage, character.hp as i32 - 1);
            let damage = cmp::max(damage, 0) as EOShort;

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

            let mut builder = StreamBuilder::new();
            builder.add_short(damage);
            builder.add_short(character.hp);
            builder.add_short(character.max_hp);

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

                builder.add_short(*other_player_id);
                builder.add_char(other.get_hp_percentage());
                builder.add_short(other_damage);
            }

            character.player.as_ref().unwrap().send(
                PacketAction::TargetOther,
                PacketFamily::Effect,
                builder.get(),
            );
        }
    }

    fn timed_drain_tp(&mut self) {
        for character in self.characters.values_mut() {
            if character.tp == 0 {
                continue;
            }

            let damage = (character.max_tp as f32 * SETTINGS.map.drain_tp_damage).floor() as i32;
            let damage = cmp::min(damage, character.tp as i32 - 1);
            let damage = cmp::max(damage, 0) as EOShort;

            character.tp -= damage;

            let mut builder = StreamBuilder::new();
            builder.add_char(1);
            builder.add_short(damage);
            builder.add_short(character.tp);
            builder.add_short(character.max_tp);

            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Effect,
                builder.get(),
            );
        }
    }
}
