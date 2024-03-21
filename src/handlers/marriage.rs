use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{MarriageOpenClientPacket, MarriageRequestClientPacket, MarriageRequestType},
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle, utils::validate_character_name};

async fn open(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let open = match MarriageOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing MarriageOpenClientPacket {}", e);
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

    map.open_law(player_id, open.npc_index, session_id);
}

async fn request(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let request = match MarriageRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing MarriageRequestClientPacket {}", e);
            return;
        }
    };

    if !validate_character_name(&request.name) {
        return;
    }

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != request.session_id {
                return;
            }
        }
        Err(_) => {
            return;
        }
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    match request.request_type {
        MarriageRequestType::MarriageApproval => {
            map.request_marriage_approval(player_id, npc_index, request.name)
        }
        MarriageRequestType::Divorce => map.request_divorce(player_id, npc_index, request.name),
        _ => {}
    }
}

pub async fn marriage(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        _ => error!("Unhandled packet Marriage_{:?}", action),
    }
}
