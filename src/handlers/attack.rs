use eolib::{data::{EoReader, EoSerialize}, protocol::net::{client::AttackUseClientPacket, PacketAction}};

use crate::player::PlayerHandle;

async fn r#use(reader: EoReader, player: PlayerHandle) {
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

    let packet = match AttackUseClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing AttackUseClientPacket {}", e);
            return;
        }
    };

    map.attack(player_id, packet.direction, packet.timestamp);
}

pub async fn attack(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Use => r#use(reader, player).await,
        _ => error!("Unhandled packet Attack_{:?}", action),
    }
}
