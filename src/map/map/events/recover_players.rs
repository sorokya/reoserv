use eolib::protocol::net::{
    server::{RecoverPlayerServerPacket, SitState},
    PacketAction, PacketFamily,
};

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

            if character.tp < character.max_tp {
                character.tp += (character.max_tp / divisor) + 1;

                if character.tp > character.max_tp {
                    character.tp = character.max_tp;
                }
            }

            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Player,
                    PacketFamily::Recover,
                    &RecoverPlayerServerPacket {
                        hp: character.hp,
                        tp: character.tp,
                    },
                );

                if original_hp != character.hp {
                    player.update_party_hp(character.get_hp_percentage());
                }
            }
        }
    }
}
