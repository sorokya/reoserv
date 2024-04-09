use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            ShopBuyClientPacket, ShopCreateClientPacket, ShopOpenClientPacket, ShopSellClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn buy(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let buy = match ShopBuyClientPacket::deserialize(&reader) {
        Ok(buy) => buy,
        Err(e) => {
            error!("Error deserializing ShopBuyClientPacket {}", e);
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

    map.buy_item(player_id, npc_index, buy.buy_item);
}

async fn create(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let create = match ShopCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing ShopCreateClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != create.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.craft_item(player_id, npc_index, create.craft_item_id);
}

async fn open(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let open = match ShopOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing ShopCreateClientPacket {}", e);
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

    map.open_shop(player_id, open.npc_index, session_id);
}

async fn sell(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let sell = match ShopSellClientPacket::deserialize(&reader) {
        Ok(sell) => sell,
        Err(e) => {
            error!("Error deserializing ShopSellClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != sell.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.sell_item(player_id, npc_index, sell.sell_item);
}

pub async fn shop(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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

    // Prevent interacting with shop when trading
    if player.is_trading().await {
        return;
    }

    match action {
        PacketAction::Buy => buy(reader, player_id, player, map).await,
        PacketAction::Create => create(reader, player_id, player, map).await,
        PacketAction::Open => open(reader, player_id, player, map).await,
        PacketAction::Sell => sell(reader, player_id, player, map).await,
        _ => error!("Unhandled packet Shop_{:?}", action),
    }
}
