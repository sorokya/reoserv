use eolib::protocol::net::PartyRequestType;

use crate::player::PartyRequest;

use super::super::World;

impl World {
    pub async fn accept_party_request(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        request_type: PartyRequestType,
    ) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        match player.get_party_request().await.expect("Failed to get party request. Timeout") {
            PartyRequest::Invite(actual_player_id) => {
                if request_type != PartyRequestType::Invite || actual_player_id != target_player_id
                {
                    return;
                }

                if self.player_in_party(actual_player_id) {
                    self.join_party(player_id, actual_player_id).await;
                } else {
                    self.create_party(actual_player_id, player_id).await;
                }
            }
            PartyRequest::Join(actual_player_id) => {
                if request_type != PartyRequestType::Join || actual_player_id != target_player_id {
                    return;
                }

                if self.player_in_party(player_id) {
                    self.join_party(actual_player_id, player_id).await;
                } else {
                    self.create_party(player_id, actual_player_id).await;
                }
            }
            _ => {}
        }
    }
}
