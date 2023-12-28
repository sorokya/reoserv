use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::PlayerRangeRequestClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match PlayerRangeRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing PlayerRangeRequestClientPacket {}", e);
            return;
        }
    };

    map.request_players(player_id, request.player_ids);
}

pub async fn player_range(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map {}", e);
            return;
        }
    };

    match action {
        PacketAction::Request => request(reader, player_id, map),
        _ => error!("Unhandled packet PlayerRange_{:?}", action),
    }
}
