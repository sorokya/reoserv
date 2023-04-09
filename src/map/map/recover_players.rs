use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{server::recover, SitState, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub async fn recover_players(&mut self) {
        for character in self.characters.values_mut() {
            if character.hp < character.max_hp {
                character.hp += (character.max_hp
                    / if character.sit_state != SitState::Stand {
                        5
                    } else {
                        10
                    })
                    + 1;

                if character.hp > character.max_hp {
                    character.hp = character.max_hp;
                }
            }

            if character.tp < character.max_tp {
                character.tp += (character.max_tp / if character.sit_state != SitState::Stand {
                    5
                } else {
                    10
                }) + 1;

                if character.tp > character.max_tp {
                    character.tp = character.max_tp;
                }
            }

            let packet = recover::Player {
                hp: character.hp,
                tp: character.tp,
                sp: 0,
            };

            debug!("{:?}", packet);

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
