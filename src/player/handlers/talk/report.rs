use eo::{
    character::AdminLevel,
    data::{Serializeable, StreamReader},
    net::packets::client::talk::Report,
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

use super::handle_command::handle_command;

pub async fn report(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut report = Report::default();
    let reader = StreamReader::new(&buf);
    report.deserialize(&reader);

    debug!("Recv: {:?}", report);

    if let Ok(character) = player.get_character().await {
        if report.message.starts_with("$") && character.admin_level != AdminLevel::Player {
            handle_command(
                &report.message[1..]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
                &character,
                player,
                world,
            )
            .await;
        } else {
            if let Ok(map) = player.get_map().await {
                let player_id = player.get_player_id().await;
                map.send_chat_message(player_id, report.message);
            }
        }
    }
}
