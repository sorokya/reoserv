use eolib::{data::{EoReader, EoSerialize}, protocol::net::{Item, client::{ChestAddClientPacket, ChestOpenClientPacket, ChestTakeClientPacket}, PacketAction}};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match ChestAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing ChestAddClientPacket {}", e);
            return;
        }
    };
    map.add_chest_item(
        player_id,
        Item {
            id: add.add_item.id,
            amount: add.add_item.amount,
        },
    );
}

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match ChestOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing ChestOpenClientPacket {}", e);
            return;
        }
    };
    map.open_chest(player_id, open.coords);
}

fn take(reader: EoReader, player_id: i32, map: MapHandle) {
    let take = match ChestTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing ChestTakeClientPacket {}", e);
            return;
        }
    };
    map.take_chest_item(player_id, take.take_item_id);
}

pub async fn chest(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
