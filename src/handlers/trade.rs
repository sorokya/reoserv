use eo::{
    data::{i32, StreamReader},
    protocol::{Item, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn request(reader: StreamReader, player_id: i32, map: MapHandle) {
    reader.get_char();

    let target_player_id = reader.get_short();
    map.request_trade(player_id, target_player_id);
}

fn accept(reader: StreamReader, player_id: i32, map: MapHandle) {
    reader.get_char();

    let target_player_id = reader.get_short();
    map.accept_trade_request(player_id, target_player_id);
}

async fn close(player: PlayerHandle, player_id: i32, map: MapHandle) {
    if let Some(interact_player_id) = player.get_interact_player_id().await {
        map.cancel_trade(player_id, interact_player_id);
    }
}

fn add(reader: StreamReader, player_id: i32, map: MapHandle) {
    let item_id = reader.get_short();
    let amount = reader.get_int();
    map.add_trade_item(
        player_id,
        Item {
            id: item_id,
            amount,
        },
    );
}

fn remove(reader: StreamReader, player_id: i32, map: MapHandle) {
    let item_id = reader.get_short();
    map.remove_trade_item(player_id, item_id);
}

fn agree(reader: StreamReader, player_id: i32, map: MapHandle) {
    let agree = reader.get_char() == 1;
    if agree {
        map.accept_trade(player_id);
    } else {
        map.unaccept_trade(player_id);
    }
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
        PacketAction::Close => close(player, player_id, map).await,
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Remove => remove(reader, player_id, map),
        PacketAction::Agree => agree(reader, player_id, map),
        _ => error!("Unhandled packet Trade_{:?}", action),
    }
}
