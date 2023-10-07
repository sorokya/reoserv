use eo::{
    data::{EOChar, EOShort, Serializeable, StreamReader},
    protocol::{
        client::shop::{Buy, Create, Open, Sell},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn buy(reader: StreamReader, player: PlayerHandle) {
    let mut request = Buy::default();
    request.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.buy_item(player_id, request.buy_item, request.session_id as EOShort);
    }
}

async fn create(reader: StreamReader, player: PlayerHandle) {
    let mut request = Create::default();
    request.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.craft_item(
            player_id,
            request.craft_item_id,
            request.session_id as EOShort,
        );
    }
}

async fn open(reader: StreamReader, player: PlayerHandle) {
    let mut request = Open::default();
    request.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.open_shop(player_id, request.npc_index as EOChar);
    }
}

async fn sell(reader: StreamReader, player: PlayerHandle) {
    let mut request = Sell::default();
    request.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.sell_item(player_id, request.sell_item, request.session_id as EOShort);
    }
}

pub async fn shop(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Buy => buy(reader, player).await,
        PacketAction::Create => create(reader, player).await,
        PacketAction::Open => open(reader, player).await,
        PacketAction::Sell => sell(reader, player).await,
        _ => error!("Unhandled packet Shop_{:?}", action),
    }
}
