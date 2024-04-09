use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{BarberBuyClientPacket, BarberOpenClientPacket},
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn open(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let open = match BarberOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing BarberOpenClientPacket {}", e);
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

    map.open_barber(player_id, open.npc_index, session_id);
}

async fn buy(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let buy = match BarberBuyClientPacket::deserialize(&reader) {
        Ok(buy) => buy,
        Err(e) => {
            error!("Error deserializing BarberBuyClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != buy.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.buy_haircut(player_id, npc_index, buy.hair_style, buy.hair_color);
}

pub async fn barber(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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

    // Prevent interacting with barber when trading
    if player.is_trading().await {
        return;
    }

    match action {
        PacketAction::Open => open(reader, player_id, player, map).await,
        PacketAction::Buy => buy(reader, player_id, player, map).await,
        _ => error!("Unhandled packet Barber_{:?}", action),
    }
}
