use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::{client::talk::Admin, AdminLevel},
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn admin(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut admin = Admin::default();
    let reader = StreamReader::new(buf);
    admin.deserialize(&reader);

    debug!("Recv: {:?}", admin);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_admin_message(character.name, admin.message);
        }
    }
}
