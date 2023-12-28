use eo::{
    data::{i32, Serializeable, StreamReader},
    protocol::{
        client::warp::{Accept, Take},
        FileType, PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn accept(reader: StreamReader, player: PlayerHandle) {
    let mut accept = Accept::default();
    accept.deserialize(&reader);
    player.accept_warp(accept.map_id, accept.session_id);
}

fn take(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);
    world.get_file(player_id, FileType::Map, take.session_id, None, true);
}

pub async fn warp(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    match action {
        PacketAction::Accept => accept(reader, player),
        PacketAction::Take => take(reader, player_id, world),
        _ => error!("Unhandled packet Warp_{:?}", action),
    }
}
