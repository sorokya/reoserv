use eolib::{data::{EoReader, EoSerialize}, protocol::net::{client::LoginRequestClientPacket, PacketAction}};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn request(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let request = match LoginRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing LoginRequestClientPacket {}", e);
            return;
        }
    };

    world.login(player_id, request.username, request.password);
}

pub async fn login(
    action: PacketAction,
    reader: EoReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Login_{:?}", action),
    }
}
