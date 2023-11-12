use eo::{
    data::{EOChar, EOShort, StreamReader},
    protocol::PacketAction,
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let npc_index = reader.get_short();
    map.open_inn(player_id, npc_index as EOChar);
}

fn reply(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let session_id = reader.get_short();
    reader.get_byte();
    let _behavior_id = reader.get_short();
    reader.get_byte();
    let answers = [
        reader.get_break_string(),
        reader.get_break_string(),
        reader.get_break_string(),
    ];

    map.request_citizenship(player_id, session_id, answers);
}

fn remove(player_id: EOShort, map: MapHandle) {
    map.remove_citizenship(player_id);
}

fn request(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let session_id = reader.get_short();
    map.request_sleep(player_id, session_id);
}

fn accept(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let session_id = reader.get_short();
    map.sleep(player_id, session_id);
}

pub async fn citizen(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
