use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::paperdoll::{Add, Remove, Request},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn add(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Add::default();
    packet.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
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

    map.equip(player_id, packet.item_id, packet.sub_loc);
}

async fn remove(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Remove::default();
    packet.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
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

    map.unequip(player_id, packet.item_id, packet.sub_loc);
}

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Request::default();
    packet.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
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

    map.request_paperdoll(player_id, packet.player_id);
}

pub async fn paperdoll(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Add => add(reader, player).await,
        PacketAction::Remove => remove(reader, player).await,
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet Paperdoll_{:?}", action),
    }
}
