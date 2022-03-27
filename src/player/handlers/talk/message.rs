use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::talk::Message,
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn message(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut message = Message::default();
    let reader = StreamReader::new(&buf);
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
