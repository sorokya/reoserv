use eo::{
    data::{EOShort, StreamReader},
    protocol::PacketAction,
};

use crate::{map::MapHandle, player::PlayerHandle};

fn request(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    reader.get_char();

    let target_player_id = reader.get_short();
    map.request_trade(player_id, target_player_id);
}

fn accept(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    reader.get_char();

    let target_player_id = reader.get_short();
    map.accept_trade_request(player_id, target_player_id);
}

fn close(player: PlayerHandle, player_id: EOShort, map: MapHandle) {
    player.set_trading(false);
    player.set_interact_player_id(None);
    map.cancel_trade(player_id);
}

pub async fn trade(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::Close => close(player, player_id, map),
        _ => error!("Unhandled packet Trade_{:?}", action),
    }
}
