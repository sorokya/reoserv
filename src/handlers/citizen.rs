use eolib::{data::{EoReader, EoSerialize}, protocol::net::{PacketAction, client::{CitizenOpenClientPacket, CitizenReplyClientPacket, CitizenRequestClientPacket, CitizenAcceptClientPacket}}};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match CitizenOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing CitizenOpenClientPacket {}", e);
            return;
        }
    };
    map.open_inn(player_id, open.npc_index);
}

fn reply(reader: EoReader, player_id: i32, map: MapHandle) {
    let reply = match CitizenReplyClientPacket::deserialize(&reader) {
        Ok(reply) => reply,
        Err(e) => {
            error!("Error deserializing CitizenReplyClientPacket {}", e);
            return;
        }
    };

    map.request_citizenship(player_id, reply.session_id, reply.answers);
}

fn remove(player_id: i32, map: MapHandle) {
    map.remove_citizenship(player_id);
}

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match CitizenRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing CitizenRequestClientPacket {}", e);
            return;
        }
    };
    map.request_sleep(player_id, request.session_id);
}

fn accept(reader: EoReader, player_id: i32, map: MapHandle) {
    let accept = match CitizenAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing CitizenAcceptClientPacket {}", e);
            return;
        }
    };
    map.sleep(player_id, accept.session_id);
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

    match action {
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Reply => reply(reader, player_id, map),
        PacketAction::Remove => remove(player_id, map),
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::Accept => accept(reader, player_id, map),
        _ => error!("Unhandled packet Citizen_{:?}", action),
    }
}
