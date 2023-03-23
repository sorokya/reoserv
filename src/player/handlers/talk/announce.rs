use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::{client::talk::Announce, AdminLevel},
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn announce(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut announce = Announce::default();
    let reader = StreamReader::new(buf);
    announce.deserialize(&reader);

    debug!("Recv: {:?}", announce);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_announcement(character.name, announce.message);
        }
    }
}
