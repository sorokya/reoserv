use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{FileType, WarpAcceptClientPacket, WarpTakeClientPacket},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

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

fn take(reader: EoReader, player: PlayerHandle) {
    let take = match WarpTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing WarpTakeClientPacket {}", e);
            return;
        }
    };

    player.get_file(FileType::Emf, take.session_id, None, true);
}

pub async fn warp(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Accept => accept(reader, player),
        PacketAction::Take => take(reader, player),
        _ => error!("Unhandled packet Warp_{:?}", action),
    }
}
