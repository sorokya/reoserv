use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            CitizenAcceptClientPacket, CitizenOpenClientPacket, CitizenReplyClientPacket,
            CitizenRequestClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn open(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let open = match CitizenOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing CitizenOpenClientPacket {}", e);
            return;
        }
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Failed to generate session id: {}", e);
            return;
        }
    };

    map.open_inn(player_id, open.npc_index, session_id);
}

async fn reply(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let reply = match CitizenReplyClientPacket::deserialize(&reader) {
        Ok(reply) => reply,
        Err(e) => {
            error!("Error deserializing CitizenReplyClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != reply.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.request_citizenship(player_id, npc_index, reply.answers);
}

async fn remove(player_id: i32, player: PlayerHandle, map: MapHandle) {
    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.remove_citizenship(player_id, npc_index);
}

async fn request(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let request = match CitizenRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing CitizenRequestClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != request.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.request_sleep(player_id, npc_index);
}

async fn accept(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let accept = match CitizenAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing CitizenAcceptClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != accept.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    let cost = match player.get_sleep_cost().await {
        Some(cost) => cost,
        None => return,
    };

    map.sleep(player_id, npc_index, cost);
}

pub async fn citizen(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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

    // Prevent interacting with citizen npc while trading
    if player.is_trading().await {
        return;
    }

    match action {
        PacketAction::Open => open(reader, player_id, player, map).await,
        PacketAction::Reply => reply(reader, player_id, player, map).await,
        PacketAction::Remove => remove(player_id, player, map).await,
        PacketAction::Request => request(reader, player_id, player, map).await,
        PacketAction::Accept => accept(reader, player_id, player, map).await,
        _ => error!("Unhandled packet Citizen_{:?}", action),
    }
}
