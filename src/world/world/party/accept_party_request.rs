use eo::data::{EOChar, EOShort};

use crate::player::PartyRequest;

use super::super::World;

const JOIN: EOChar = 0;
const INVITE: EOChar = 1;

impl World {
    pub async fn accept_party_request(
        &mut self,
        player_id: EOShort,
        target_player_id: EOShort,
        request_type: EOChar,
    ) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        match player.get_party_request().await {
            PartyRequest::Invite(actual_player_id) => {
                if request_type != INVITE || actual_player_id != target_player_id {
                    return;
                }

                if self.player_in_party(actual_player_id) {
                    self.join_party(player_id, actual_player_id);
                } else {
                    self.create_party(actual_player_id, player_id).await;
                }
            }
            PartyRequest::Join(actual_player_id) => {
                if request_type != JOIN || actual_player_id != target_player_id {
                    return;
                }

                if self.player_in_party(player_id) {
                    self.join_party(actual_player_id, player_id);
                } else {
                    self.create_party(player_id, actual_player_id).await;
                }
            }
            _ => {}
        }
    }
}