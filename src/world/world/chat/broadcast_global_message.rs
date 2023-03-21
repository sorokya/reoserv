use eo::{
    data::{EOShort, Serializeable},
    protocol::{server::talk, PacketAction, PacketFamily},
};

use crate::player::ClientState;

use super::super::World;

impl World {
    pub async fn broadcast_global_message(
        &self,
        target_player_id: EOShort,
        name: &str,
        message: &str,
    ) {
        let packet = talk::Msg {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let buf = packet.serialize();
        for player in self.players.values() {
            let state = player.get_state().await;

            if state.is_err() {
                continue;
            }

            let state = state.unwrap();

            let player_id = player.get_player_id().await;

            if player_id.is_err() {
                continue;
            }

            let player_id = player_id.unwrap();

            if state == ClientState::Playing && player_id != target_player_id {
                player.send(PacketAction::Msg, PacketFamily::Talk, buf.clone());
            }
        }
    }
}
