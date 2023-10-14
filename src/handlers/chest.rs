use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::{
        client::chest::{Add, Open, Take},
        Item, PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Add::default();
    packet.deserialize(&reader);
    map.add_chest_item(
        player_id,
        Item {
            id: packet.add_item.id,
            amount: packet.add_item.amount,
        },
    );
}

fn open(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Open::default();
    packet.deserialize(&reader);
    map.open_chest(player_id, packet.coords);
}

fn take(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Take::default();
    packet.deserialize(&reader);
    map.take_chest_item(player_id, packet.take_item_id);
}

pub async fn chest(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Chest_{:?}", action),
    }
}
