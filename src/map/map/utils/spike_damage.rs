use std::cmp;

use eolib::{
    data::EoWriter,
    protocol::net::{PacketAction, PacketFamily},
};

use crate::SETTINGS;

use super::super::Map;

const EFFECT_SPIKE: i32 = 2;

impl Map {
    pub fn spike_damage(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let damage = (character.max_hp as f32 * SETTINGS.world.spike_damage).floor() as i32;
        let damage = cmp::min(damage, character.hp);

        character.hp -= damage;

        let hp_percentage = character.get_hp_percentage();

        let mut writer = EoWriter::new();
        writer.add_char(EFFECT_SPIKE);
        writer.add_short(damage);
        writer.add_short(character.hp);
        writer.add_short(character.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.player.as_ref().unwrap().send(
            PacketAction::Spec,
            PacketFamily::Effect,
            writer.to_byte_array(),
        );

        let mut writer = EoWriter::new();
        writer.add_short(player_id);
        writer.add_char(hp_percentage);
        writer.add_char(if character.hp == 0 { 1 } else { 0 });
        writer.add_three(damage as i32);

        self.send_buf_near_player(
            player_id,
            PacketAction::Admin,
            PacketFamily::Effect,
            writer.to_byte_array(),
        );

        if character.hp == 0 {
            character.player.as_ref().unwrap().die();
        }
    }
}
