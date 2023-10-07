use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::item::{Drop, Get, Junk, Use},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn drop(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Drop::default();
    packet.deserialize(&reader);

    let player_id = player.get_player_id().await;

    if let Err(e) = player_id {
        error!("Failed to get player id: {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;

    if let Err(e) = map {
        error!("Failed to get map: {}", e);
        return;
    }

    map.unwrap()
        .drop_item(player_id, packet.drop_item, packet.coords);
}

async fn get(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Get::default();
    packet.deserialize(&reader);

    let player_id = player.get_player_id().await;

    if let Err(e) = player_id {
        error!("Failed to get player id: {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;

    if let Err(e) = map {
        error!("Failed to get map: {}", e);
        return;
    }

    map.unwrap().get_item(player_id, packet.take_item_index);
}

async fn junk(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Junk::default();
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

    map.junk_item(player_id, packet.junk_item.id, packet.junk_item.amount);
}

async fn r#use(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Use::default();
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

    map.use_item(player_id, packet.use_item_id);
}

pub async fn item(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Drop => drop(reader, player).await,
        PacketAction::Get => get(reader, player).await,
        PacketAction::Junk => junk(reader, player).await,
        PacketAction::Use => r#use(reader, player).await,
        _ => error!("Unhandled packet Item_{:?}", action),
    }
}
