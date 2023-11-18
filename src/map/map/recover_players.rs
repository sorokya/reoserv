use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{server::recover, PacketAction, PacketFamily, SitState},
};

use super::Map;

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

            let packet = recover::Player {
                hp: character.hp,
                tp: character.tp,
                sp: 0,
            };

            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);
            character.player.as_ref().unwrap().send(
                PacketAction::Player,
                PacketFamily::Recover,
                builder.get(),
            );

            // TODO: party recovery
        }
    }
}
