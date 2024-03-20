use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::PriestOpenClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn open(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let open = match PriestOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing PriestOpenClientPacket {}", e);
            return;
        }
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Error generating session id: {}", e);
            return;
        }
    };

    map.open_priest(player_id, open.npc_index, session_id);
}

pub async fn priest(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map: {}", e);
            return;
        }
    };
    match action {
        PacketAction::Open => open(reader, player, player_id, map).await,
        _ => error!("Unhandled packet Priest_{:?}", action),
    }
}
