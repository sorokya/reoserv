use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::emote::Report, PacketAction},
};

use crate::player::PlayerHandle;

async fn report(reader: StreamReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let mut report = Report::default();
    report.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        map.emote(player_id, report.emote);
    }
}

pub async fn emote(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Report => report(reader, player).await,
        _ => error!("Unhandled packet Emote_{:?}", action),
    }
}
