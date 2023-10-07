use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::talk::Msg,
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn message(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut message = Msg::default();
    message.deserialize(&reader);

    debug!("Recv: {:?}", message);

    if let Ok(character) = player.get_character().await {
        world.broadcast_global_message(
            character.player_id.unwrap(),
            character.name,
            message.message,
        )
    }
}
