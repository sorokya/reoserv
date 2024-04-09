use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkMsgServerPacket, PacketAction, PacketFamily},
};

use crate::{player::ClientState, LANG};

use super::super::World;

impl World {
    // TODO: make this sync
    pub async fn broadcast_global_message(&self, player_id: i32, name: &str, message: &str) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        if self.global_locked {
            player.send(
                PacketAction::Msg,
                PacketFamily::Talk,
                &TalkMsgServerPacket {
                    player_name: "Server".to_string(),
                    message: LANG.global_locked.to_owned(),
                },
            );
            return;
        }

        let packet = TalkMsgServerPacket {
            player_name: name.to_string(),
            message: message.to_string(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkMsgServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            let state = match player.get_state().await {
                Ok(state) => state,
                Err(e) => {
                    error!("Failed to get state: {}", e);
                    continue;
                }
            };

            let other_player_id = match player.get_player_id().await {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to get player_id: {}", e);
                    continue;
                }
            };

            if state == ClientState::InGame && player_id != other_player_id {
                player.send_buf(PacketAction::Msg, PacketFamily::Talk, buf.clone());
            }
        }
    }
}
