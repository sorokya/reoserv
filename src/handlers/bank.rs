use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{BankAddClientPacket, BankOpenClientPacket, BankTakeClientPacket},
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn add(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let add = match BankAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing BankAddClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != add.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.deposit_gold(player_id, npc_index, add.amount);
}

async fn open(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let open = match BankOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing BankOpenClientPacket {}", e);
            return;
        }
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Failed to generate session id: {}", e);
            return;
        }
    };

    map.open_bank(player_id, open.npc_index, session_id);
}

async fn take(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let take = match BankTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing BankTakeClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != take.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.withdraw_gold(player_id, npc_index, take.amount);
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

    // Prevent interacting with bank when trading
    if player.is_trading().await {
        return;
    }

    match action {
        PacketAction::Add => add(reader, player_id, player, map).await,
        PacketAction::Open => open(reader, player_id, player, map).await,
        PacketAction::Take => take(reader, player_id, player, map).await,
        _ => error!("Unhandled packet Bank_{:?}", action),
    }
}
