use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::emote::Report,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn report(
    buf: Bytes,
    player: PlayerHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut report = Report::default();
    let reader = StreamReader::new(buf);
    report.deserialize(&reader);

    debug!("Recv: {:?}", report);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.emote(player_id, report.emote);
    }

    Ok(())
}
