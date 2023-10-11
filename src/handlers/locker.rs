use eo::{
    data::{EOShort, StreamReader},
    protocol::{Item, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(player_id: EOShort, map: MapHandle) {
    map.open_locker(player_id);
}

fn add(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    reader.seek(2);

    let item = Item {
        id: reader.get_short(),
        amount: reader.get_three(),
    };

    map.add_locker_item(player_id, item);
}

pub async fn locker(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Open => open(player_id, map),
        PacketAction::Add => add(reader, player_id, map),
        _ => error!("Unhandled packet Locker_{:?}", action),
    }
}
