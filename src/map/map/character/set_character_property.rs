use eolib::protocol::{
    AdminLevel, Gender,
    net::{
        PacketAction, PacketFamily,
        server::{AvatarRemoveServerPacket, NearbyInfo, PlayersAgreeServerPacket},
    },
};

use crate::{CLASS_DB, SETTINGS, utils::validate_character_name};

use super::super::Map;

impl Map {
    pub fn set_character_property(&mut self, player_id: i32, property: String, value: String) {
        let visible = {
            let character = match self.characters.get_mut(&player_id) {
                Some(character) => character,
                None => return,
            };

            match property.as_str() {
                "title" => {
                    let title_length = value.chars().count();
                    if title_length > 0
                        && title_length <= SETTINGS.load().character.max_title_length
                    {
                        character.title = Some(value);
                    } else {
                        character.title = None;
                    }
                    false
                }
                "fiance" => {
                    let name = value.to_lowercase();
                    if validate_character_name(&name) {
                        character.fiance = Some(name);
                    } else {
                        character.fiance = None;
                    }
                    false
                }
                "partner" => {
                    let name = value.to_lowercase();
                    if validate_character_name(&name) {
                        character.partner = Some(name);
                    } else {
                        character.partner = None;
                    }
                    false
                }
                "home" => {
                    character.home = value;
                    false
                }
                "gender" => {
                    let gender = match value.to_lowercase().as_str() {
                        "male" | "m" => Gender::Male,
                        _ => Gender::Female,
                    };
                    if character.gender != gender {
                        character.gender = gender;
                        true
                    } else {
                        false
                    }
                }
                "admin" => {
                    if let Ok(admin_level) = value.parse::<i32>()
                        && (0..=5).contains(&admin_level)
                    {
                        character.admin_level = AdminLevel::from(admin_level);
                    }
                    false
                }
                "hairstyle" => {
                    if let Ok(hair_style) = value.parse::<i32>()
                        && (0..=SETTINGS.load().character.max_hair_style).contains(&hair_style)
                        && character.hair_style != hair_style
                    {
                        character.hair_style = hair_style;
                        true
                    } else {
                        false
                    }
                }
                "haircolor" => {
                    if let Ok(hair_color) = value.parse::<i32>()
                        && (0..=SETTINGS.load().character.max_hair_color).contains(&hair_color)
                        && character.hair_color != hair_color
                    {
                        character.hair_color = hair_color;
                        true
                    } else {
                        false
                    }
                }
                "skin" => {
                    if let Ok(skin) = value.parse::<i32>()
                        && (0..=SETTINGS.load().character.max_skin).contains(&skin)
                        && character.skin != skin
                    {
                        character.skin = skin;
                        true
                    } else {
                        false
                    }
                }
                "class" => {
                    if let Ok(class_id) = value.parse::<i32>()
                        && CLASS_DB
                            .load()
                            .classes
                            .iter()
                            .enumerate()
                            .any(|(id, _)| id == (class_id - 1) as usize)
                    {
                        character.class = class_id;
                    }
                    false
                }
                _ => false,
            }
        };

        if visible {
            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            self.send_packet_near(
                &character.coords,
                PacketAction::Remove,
                PacketFamily::Avatar,
                AvatarRemoveServerPacket {
                    player_id,
                    warp_effect: None,
                },
            );
            self.send_packet_near(
                &character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                PlayersAgreeServerPacket {
                    nearby: NearbyInfo {
                        characters: vec![character.to_map_info()],
                        npcs: Vec::new(),
                        items: Vec::new(),
                    },
                },
            );
        }
    }
}
