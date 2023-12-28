use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            ItemDropClientPacket, ItemGetClientPacket, ItemJunkClientPacket, ItemUseClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn drop(reader: EoReader, player_id: i32, map: MapHandle) {
    let drop = match ItemDropClientPacket::deserialize(&reader) {
        Ok(drop) => drop,
        Err(e) => {
            error!("Error deserializing ItemDropClientPacket {}", e);
            return;
        }
    };
    map.drop_item(player_id, drop.item, drop.coords);
}

fn get(reader: EoReader, player_id: i32, map: MapHandle) {
    let get = match ItemGetClientPacket::deserialize(&reader) {
        Ok(get) => get,
        Err(e) => {
            error!("Error deserializing ItemGetClientPacket {}", e);
            return;
        }
    };
    map.get_item(player_id, get.item_index);
}

fn junk(reader: EoReader, player_id: i32, map: MapHandle) {
    let junk = match ItemJunkClientPacket::deserialize(&reader) {
        Ok(junk) => junk,
        Err(e) => {
            error!("Error deserializing ItemJunkClientPacket {}", e);
            return;
        }
    };
    map.junk_item(player_id, junk.item.id, junk.item.amount);
}

fn r#use(reader: EoReader, player_id: i32, map: MapHandle) {
    let packet = match ItemUseClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing ItemUseClientPacket {}", e);
            return;
        }
    };
    map.use_item(player_id, packet.item_id);
}

pub async fn item(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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

    match action {
        PacketAction::Drop => drop(reader, player_id, map),
        PacketAction::Get => get(reader, player_id, map),
        PacketAction::Junk => junk(reader, player_id, map),
        PacketAction::Use => r#use(reader, player_id, map),
        _ => error!("Unhandled packet Item_{:?}", action),
    }
}
