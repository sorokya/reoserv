use eolib::protocol::net::server::{BigCoords, CharacterMapInfo};

use crate::utils::pad_string;

use super::Character;

impl Character {
    pub fn to_map_info(&self) -> CharacterMapInfo {
        CharacterMapInfo {
            name: self.name.clone(),
            player_id: self.player_id.expect("Character has no player id"),
            map_id: self.map_id,
            coords: BigCoords {
                x: self.coords.x,
                y: self.coords.y,
            },
            direction: self.direction,
            class_id: self.class,
            guild_tag: match self.guild_tag {
                Some(ref tag) => pad_string(tag, 3),
                None => "   ".to_string(),
            },
            level: self.level,
            gender: self.gender,
            hair_style: self.hair_style,
            hair_color: self.hair_color,
            skin: self.skin,
            max_hp: self.max_hp,
            hp: self.hp,
            max_tp: self.max_tp,
            tp: self.tp,
            equipment: self.get_equipment_map_info(),
            sit_state: self.sit_state,
            invisible: self.hidden,
            warp_effect: None,
        }
    }
}
