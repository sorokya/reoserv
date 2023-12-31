use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            EffectAdminServerPacket, EffectSpecServerPacket,
            EffectSpecServerPacketMapDamageTypeData, EffectSpecServerPacketMapDamageTypeDataSpikes,
            MapDamageType,
        },
        PacketAction, PacketFamily,
    },
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

        let packet = EffectSpecServerPacket {
            map_damage_type: MapDamageType::Spikes,
            map_damage_type_data: Some(EffectSpecServerPacketMapDamageTypeData::Spikes(
                EffectSpecServerPacketMapDamageTypeDataSpikes {
                    hp_damage: damage,
                    hp: character.hp,
                    max_hp: character.max_hp,
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize EffectSpecServerPacket: {}", e);
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.player.as_ref().unwrap().send(
            PacketAction::Spec,
            PacketFamily::Effect,
            writer.to_byte_array(),
        );

        let packet = EffectAdminServerPacket {
            player_id,
            hp_percentage,
            died: character.hp == 0,
            damage,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize EffectAdminServerPacket: {}", e);
            return;
        }

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
