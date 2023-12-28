use eolib::{data::{EoReader, EoSerialize}, protocol::net::{PacketAction, client::{BankAddClientPacket, BankOpenClientPacket, BankTakeClientPacket}}};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match BankAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing BankAddClientPacket {}", e);
            return;
        }
    };

    if add.amount == 0 {
        return;
    }

    map.deposit_gold(player_id, add.session_id, add.amount);
}

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match BankOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing BankOpenClientPacket {}", e);
            return;
        }
    };

    map.open_bank(player_id, open.npc_index);
}

fn take(reader: EoReader, player_id: i32, map: MapHandle) {
    let take = match BankTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing BankTakeClientPacket {}", e);
            return;
        }
    };

    if take.amount == 0 {
        return;
    }

    map.withdraw_gold(player_id, take.session_id, take.amount);
}

pub async fn bank(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Bank_{:?}", action),
    }
}
