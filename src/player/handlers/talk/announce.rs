use eo::{
    character::AdminLevel,
    data::{EOChar, Serializeable, StreamReader},
    net::packets::client::talk::Announce,
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn announce(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut announce = Announce::default();
    let reader = StreamReader::new(&buf);
    announce.deserialize(&reader);

    debug!("Recv: {:?}", announce);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_announcement(character.name, announce.message);
        }
    }
}
