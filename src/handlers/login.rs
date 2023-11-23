use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::login::Request, PacketAction},
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let mut request = Request::default();
    request.deserialize(&reader);

    world.login(player_id, request.username, request.password);
}

pub async fn login(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Login_{:?}", action),
    }
}
