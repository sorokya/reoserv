use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::{client::talk::Admin, AdminLevel},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn admin(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut admin = Admin::default();
    admin.deserialize(&reader);

    debug!("Recv: {:?}", admin);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_admin_message(character.name, admin.message);
        }
    }
}
