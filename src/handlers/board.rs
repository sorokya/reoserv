use eo::{
    data::{EOShort, StreamReader},
    protocol::PacketAction,
};

use crate::{map::MapHandle, player::PlayerHandle};

fn create(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let _board_id = reader.get_short();
    reader.seek(1);
    let subject = reader.get_break_string();
    let body = reader.get_break_string();
    map.create_board_post(player_id, subject, body);
}

fn open(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let board_id = reader.get_short();
    map.open_board(player_id, board_id + 1);
}

fn take(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let _board_id = reader.get_short();
    let post_id = reader.get_short();
    map.view_board_post(player_id, post_id);
}

pub async fn board(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Create => create(reader, player_id, map),
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Board_{:?}", action),
    }
}
