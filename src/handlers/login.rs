use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::LoginRequestClientPacket, PacketAction},
};

use crate::player::PlayerHandle;

async fn request(reader: EoReader, player: PlayerHandle) {
    let request = match LoginRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing LoginRequestClientPacket {}", e);
            return;
        }
    };

    player.login(request.username, request.password);
}

pub async fn login(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet Login_{:?}", action),
    }
}
