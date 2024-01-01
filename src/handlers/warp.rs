use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{FileType, WarpAcceptClientPacket, WarpTakeClientPacket},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn accept(reader: EoReader, player: PlayerHandle) {
    let accept = match WarpAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing WarpAcceptClientPacket {}", e);
            return;
        }
    };

    player.accept_warp(accept.map_id, accept.session_id);
}

fn take(reader: EoReader, player_id: i32, world: WorldHandle) {
    let take = match WarpTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing WarpTakeClientPacket {}", e);
            return;
        }
    };

    world.get_file(player_id, FileType::Emf, take.session_id, None, true);
}

pub async fn warp(
    action: PacketAction,
    reader: EoReader,
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
