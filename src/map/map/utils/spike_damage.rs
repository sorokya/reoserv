use std::cmp;

use eolib::protocol::net::{
    server::{
        EffectAdminServerPacket, EffectSpecServerPacket, EffectSpecServerPacketMapDamageTypeData,
        EffectSpecServerPacketMapDamageTypeDataSpikes, MapDamageType,
    },
    PacketAction, PacketFamily,
};

use crate::SETTINGS;

use super::super::Map;

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

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        self.send_packet_near_player(
            player_id,
            PacketAction::Admin,
            PacketFamily::Effect,
            &EffectAdminServerPacket {
                player_id,
                hp_percentage,
                died: character.hp == 0,
                damage,
            },
        );

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Spec,
                PacketFamily::Effect,
                &EffectSpecServerPacket {
                    map_damage_type: MapDamageType::Spikes,
                    map_damage_type_data: Some(EffectSpecServerPacketMapDamageTypeData::Spikes(
                        EffectSpecServerPacketMapDamageTypeDataSpikes {
                            hp_damage: damage,
                            hp: character.hp,
                            max_hp: character.max_hp,
                        },
                    )),
                },
            );

            player.update_party_hp(character.get_hp_percentage());

            if character.hp == 0 {
                player.die();
            }
        }
    }
}
