use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::InitInitClientPacket, PacketAction},
};

use crate::player::PlayerHandle;

fn request(reader: EoReader, player: PlayerHandle) {
    let packet = match InitInitClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            player.close(format!("Failed to deserialize InitInitClientPacket: {}", e));
            return;
        }
    };

    player.begin_handshake(packet.challenge, packet.hdid, packet.version);
}

pub async fn init(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Init => request(reader, player),
        _ => error!("Unhandled packet Init_{:?}", action),
    }
}
