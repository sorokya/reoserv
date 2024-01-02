use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::BarberOpenClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match BarberOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing BarberOpenClientPacket {}", e);
            return;
        }
    };

    map.open_barber(player_id, open.npc_index);
}

pub async fn barber(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Open => open(reader, player_id, map),
        _ => error!("Unhandled packet Barber_{:?}", action),
    }
}
