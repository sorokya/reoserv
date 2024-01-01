use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            TradeAcceptClientPacket, TradeAddClientPacket, TradeAgreeClientPacket,
            TradeRemoveClientPacket, TradeRequestClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match TradeRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing TradeRequestClientPacket {}", e);
            return;
        }
    };
    map.request_trade(player_id, request.player_id);
}

fn accept(reader: EoReader, player_id: i32, map: MapHandle) {
    let accept = match TradeAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing TradeAcceptClientPacket {}", e);
            return;
        }
    };

    map.accept_trade_request(player_id, accept.player_id);
}

async fn close(player: PlayerHandle, player_id: i32, map: MapHandle) {
    if let Some(interact_player_id) = player.get_interact_player_id().await {
        map.cancel_trade(player_id, interact_player_id);
    }
}

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match TradeAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing TradeAddClientPacket {}", e);
            return;
        }
    };

    map.add_trade_item(player_id, add.add_item);
}

fn remove(reader: EoReader, player_id: i32, map: MapHandle) {
    let remove = match TradeRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing TradeRemoveClientPacket {}", e);
            return;
        }
    };

    map.remove_trade_item(player_id, remove.item_id);
}

fn agree(reader: EoReader, player_id: i32, map: MapHandle) {
    let agree = match TradeAgreeClientPacket::deserialize(&reader) {
        Ok(agree) => agree,
        Err(e) => {
            error!("Error deserializing TradeAgreeClientPacket {}", e);
            return;
        }
    };

    if agree.agree {
        map.agree_trade(player_id);
    } else {
        map.disagree_trade(player_id);
    }
}

pub async fn trade(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::Close => close(player, player_id, map).await,
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Remove => remove(reader, player_id, map),
        PacketAction::Agree => agree(reader, player_id, map),
        _ => error!("Unhandled packet Trade_{:?}", action),
    }
}
