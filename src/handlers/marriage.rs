use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::MarriageOpenClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match MarriageOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing MarriageOpenClientPacket {}", e);
            return;
        }
    };

    map.open_law(player_id, open.npc_index);
}

pub async fn marriage(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player_id {}", e);
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
        PacketAction::Open => open(reader, player_id, map),
        _ => error!("Unhandled packet Marriage_{:?}", action),
    }
}
