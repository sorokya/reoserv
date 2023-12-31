use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkMsgServerPacket, PacketAction, PacketFamily},
};

use crate::{player::ClientState, LANG};

use super::super::World;

impl World {
    // TODO: make this sync
    pub async fn broadcast_global_message(&self, target_player_id: i32, name: &str, message: &str) {
        let player = match self.players.get(&target_player_id) {
            Some(player) => player,
            None => return,
        };

        if self.global_locked {
            let mut writer = EoWriter::new();
            writer.add_string("Server");
            writer.add_byte(0xff);
            writer.add_string(&LANG.global_locked);
            player.send(
                PacketAction::Msg,
                PacketFamily::Talk,
                writer.to_byte_array(),
            );
            return;
        }

        let packet = TalkMsgServerPacket {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();
        packet.serialize(&mut writer);
        let buf = writer.to_byte_array();
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
