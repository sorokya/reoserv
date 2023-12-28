use eolib::{data::{EoReader, EoSerialize}, protocol::net::{client::{PaperdollAddClientPacket, PaperdollRemoveClientPacket, PaperdollRequestClientPacket}, PacketAction}};

use crate::{player::PlayerHandle, map::MapHandle};

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match PaperdollAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing PaperdollAddClientPacket {}", e);
            return;
        }
    };

    map.equip(player_id, add.item_id, add.sub_loc);
}

fn remove(reader: EoReader, player_id: i32, map: MapHandle) {
    let remove = match PaperdollRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing PaperdollRemoveClientPacket {}", e);
            return;
        }
    };

    map.unequip(player_id, remove.item_id, remove.sub_loc);
}

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match PaperdollRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing PaperdollRequestClientPacket {}", e);
            return;
        }
    };

    map.request_paperdoll(player_id, request.player_id);
}

pub async fn paperdoll(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Remove => remove(reader, player_id, map),
        PacketAction::Request => request(reader, player_id, map),
        _ => error!("Unhandled packet Paperdoll_{:?}", action),
    }
}
