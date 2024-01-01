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

fn buy(reader: EoReader, player_id: i32, map: MapHandle) {
    let buy = match ShopBuyClientPacket::deserialize(&reader) {
        Ok(buy) => buy,
        Err(e) => {
            error!("Error deserializing ShopBuyClientPacket {}", e);
            return;
        }
    };

    map.buy_item(player_id, buy.buy_item, buy.session_id);
}

fn create(reader: EoReader, player_id: i32, map: MapHandle) {
    let create = match ShopCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing ShopCreateClientPacket {}", e);
            return;
        }
    };

    map.craft_item(player_id, create.craft_item_id, create.session_id);
}

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match ShopOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing ShopCreateClientPacket {}", e);
            return;
        }
    };

    map.open_shop(player_id, open.npc_index);
}

fn sell(reader: EoReader, player_id: i32, map: MapHandle) {
    let sell = match ShopSellClientPacket::deserialize(&reader) {
        Ok(sell) => sell,
        Err(e) => {
            error!("Error deserializing ShopSellClientPacket {}", e);
            return;
        }
    };

    map.sell_item(player_id, sell.sell_item, sell.session_id);
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

    match action {
        PacketAction::Buy => buy(reader, player_id, map),
        PacketAction::Create => create(reader, player_id, map),
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Sell => sell(reader, player_id, map),
        _ => error!("Unhandled packet Shop_{:?}", action),
    }
}
