use eolib::{protocol::net::{server::{SitState, RecoverPlayerServerPacket}, PacketAction, PacketFamily}, data::{EoWriter, EoSerialize}};

use super::super::Map;

impl Map {
    pub async fn recover_players(&mut self) {
        for character in self.characters.values_mut() {
            let divisor = match character.sit_state {
                SitState::Stand => 5,
                _ => 10,
            };

            let original_hp = character.hp;
            if character.hp < character.max_hp {
                character.hp += (character.max_hp / divisor) + 1;

                if character.hp > character.max_hp {
                    character.hp = character.max_hp;
                }
            }

            if original_hp != character.hp {
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .update_party_hp(character.get_hp_percentage());
            }

            if character.tp < character.max_tp {
                character.tp += (character.max_tp / divisor) + 1;

                if character.tp > character.max_tp {
                    character.tp = character.max_tp;
                }
            }

            let packet = RecoverPlayerServerPacket {
                hp: character.hp,
                tp: character.tp,
            };

            let mut writer = EoWriter::new();
            packet.serialize(&mut writer);
            character.player.as_ref().unwrap().send(
                PacketAction::Player,
                PacketFamily::Recover,
                writer.to_byte_array(),
            );

            // TODO: party recovery
        }
    }
}