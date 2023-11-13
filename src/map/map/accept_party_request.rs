use eo::data::{EOChar, EOShort};

use crate::player::PartyRequest;

use super::Map;

const JOIN: EOChar = 0;
const INVITE: EOChar = 1;

impl Map {
    pub async fn accept_party_request(
        &self,
        player_id: EOShort,
        target_player_id: EOShort,
        request_type: EOChar,
    ) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let world = self.world.clone();
        match player.get_party_request().await {
            PartyRequest::Invite(actual_player_id) => {
                if request_type != INVITE || actual_player_id != target_player_id {
                    return;
                }

                tokio::spawn(async move {
                    if world.player_in_party(actual_player_id).await {
                        world.join_party(player_id, actual_player_id);
                    } else {
                        world.create_party(actual_player_id, player_id);
                    }
                });
            }
            PartyRequest::Join(actual_player_id) => {
                if request_type != JOIN || actual_player_id != target_player_id {
                    return;
                }

                tokio::spawn(async move {
                    if world.player_in_party(player_id).await {
                        world.join_party(actual_player_id, player_id);
                    } else {
                        world.create_party(player_id, actual_player_id);
                    }
                });
            }
            _ => {}
        }
    }
}
