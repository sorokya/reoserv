use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{LockerAddClientPacket, LockerTakeClientPacket},
        Item, PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match LockerAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing LockerAddClientPacket {}", e);
            return;
        }
    };

    map.add_locker_item(
        player_id,
        Item {
            id: add.deposit_item.id,
            amount: add.deposit_item.amount,
        },
    );
}

fn buy(player_id: i32, map: MapHandle) {
    map.upgrade_locker(player_id);
}

fn open(player_id: i32, map: MapHandle) {
    map.open_locker(player_id);
}

fn take(reader: EoReader, player_id: i32, map: MapHandle) {
    let take = match LockerTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing LockerTakeClientPacket {}", e);
            return;
        }
    };

    map.take_locker_item(player_id, take.take_item_id);
}

pub async fn locker(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Buy => buy(player_id, map),
        PacketAction::Open => open(player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Locker_{:?}", action),
    }
}
