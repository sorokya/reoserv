use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{PriestAcceptClientPacket, PriestOpenClientPacket, PriestRequestClientPacket},
        PacketAction,
    },
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

async fn request(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let request = match PriestRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing PriestRequestClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != request.session_id {
                return;
            }
        }
        Err(_) => {
            return;
        }
    };

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.request_wedding(player_id, npc_index, request.name);
}

async fn accept(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let accept = match PriestAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing PriestAcceptClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != accept.session_id {
                return;
            }
        }
        Err(_) => {
            return;
        }
    }

    map.accept_wedding_request(player_id);
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
        PacketAction::Request => request(reader, player, player_id, map).await,
        PacketAction::Accept => accept(reader, player, player_id, map).await,
        _ => error!("Unhandled packet Priest_{:?}", action),
    }
}
