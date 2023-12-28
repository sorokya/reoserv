use eolib::{data::{EoReader, EoSerialize}, protocol::net::{PacketAction, client::EmoteReportClientPacket}};

use crate::player::PlayerHandle;

async fn report(reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let report = match EmoteReportClientPacket::deserialize(&reader) {
        Ok(report) => report,
        Err(e) => {
            error!("Error deserializing EmoteReportClientPacket {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.emote(player_id, report.emote);
    }
}

pub async fn emote(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Report => report(reader, player).await,
        _ => error!("Unhandled packet Emote_{:?}", action),
    }
}
