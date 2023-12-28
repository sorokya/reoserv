use eo::{
    data::{i32, EOInt, i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::{player::PartyRequest, utils::in_client_range, SETTINGS};

use super::super::Map;

const IN_OTHER_PARTY: i32 = 0;
const IN_YOUR_PARTY: i32 = 1;
const PARTY_FULL: i32 = 2;

impl Map {
    pub async fn party_request(&self, target_player_id: i32, request: PartyRequest) {
        let player_id = match request {
            PartyRequest::Join(id) => id,
            PartyRequest::Invite(id) => id,
            _ => return,
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if !in_client_range(&character.coords, &target_character.coords) {
            return;
        }

        // Check if player already in a party
        if let Some(party) = self.world.get_player_party(target_player_id).await {
            let mut builder = StreamBuilder::new();

            let reply = match request {
                PartyRequest::Join(_) => {
                    if party.members.contains(&player_id) {
                        Some(IN_YOUR_PARTY)
                    } else {
                        None
                    }
                }
                PartyRequest::Invite(_) => {
                    if party.members.contains(&player_id) {
                        Some(IN_YOUR_PARTY)
                    } else {
                        Some(IN_OTHER_PARTY)
                    }
                }
                _ => None,
            };

            if let Some(reply) = reply {
                builder.add_char(reply);
                builder.add_string(&target_character.name);
                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Party,
                    builder.get(),
                );

                return;
            }
        }

        // Check if party is full
        if let Some(party) = self
            .world
            .get_player_party(match request {
                PartyRequest::Join(_) => target_player_id,
                PartyRequest::Invite(_) => player_id,
                _ => return,
            })
            .await
        {
            if party.members.len() as EOInt >= SETTINGS.limits.max_party_size {
                let mut builder = StreamBuilder::new();
                builder.add_char(PARTY_FULL);
                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Party,
                    builder.get(),
                );

                return;
            }
        }

        let target = match target_character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        target.set_party_request(request);

        let mut builder = StreamBuilder::new();
        builder.add_char(match request {
            PartyRequest::Invite(_) => 1,
            PartyRequest::Join(_) => 0,
            _ => return,
        });
        builder.add_short(player_id);
        builder.add_string(&character.name);

        target.send(PacketAction::Request, PacketFamily::Party, builder.get());
    }
}
