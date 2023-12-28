use eo::{
    data::{i32, i32, StreamReader},
    protocol::PacketAction,
};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: StreamReader, player_id: i32, map: MapHandle) {
    let amount = reader.get_int();
    if amount == 0 {
        return;
    }

    let session_id = reader.get_three();

    map.deposit_gold(player_id, session_id, amount);
}

fn open(reader: StreamReader, player_id: i32, map: MapHandle) {
    let npc_index = reader.get_short();
    map.open_bank(player_id, npc_index as i32);
}

fn take(reader: StreamReader, player_id: i32, map: MapHandle) {
    let amount = reader.get_int();
    if amount == 0 {
        return;
    }

    let session_id = reader.get_three();

    map.withdraw_gold(player_id, session_id, amount);
}

pub async fn bank(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Bank_{:?}", action),
    }
}
