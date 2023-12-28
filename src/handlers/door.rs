use eolib::{data::{EoReader, EoSerialize}, protocol::net::{PacketAction, client::DoorOpenClientPacket}};

use crate::player::PlayerHandle;

async fn open(reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let open = match DoorOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing DoorOpenClientPacket {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.open_door(player_id, open.coords);
    }
}

pub async fn door(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Open => open(reader, player).await,
        _ => error!("Unhandled packet Door_{:?}", action),
    }
}
