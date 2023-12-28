use eo::{
    data::i32,
    protocol::{BigCoords, CharacterMapInfo},
};

use super::Character;

impl Character {
    pub fn to_map_info(&self) -> CharacterMapInfo {
        CharacterMapInfo {
            name: self.name.clone(),
            id: self.player_id.expect("Character has no player id"),
            map_id: self.map_id,
            coords: BigCoords {
                x: self.coords.x.into(),
                y: self.coords.y.into(),
            },
            direction: self.direction,
            class_id: self.class,
            guild_tag: match self.guild_tag {
                Some(ref tag) => tag.to_string(),
                None => String::new(),
            },
            level: self.level,
            gender: self.gender,
            hairstyle: self.hair_style as i32,
            haircolor: self.hair_color as i32,
            skin_id: self.skin,
            max_hp: self.max_hp,
            hp: self.hp,
            max_tp: self.max_tp,
            tp: self.tp,
            paperdoll: self.get_paperdoll_b000a0hsw(),
            sit_state: self.sit_state,
            invisible: i32::from(self.hidden),
            animation: None,
        }
    }
}
