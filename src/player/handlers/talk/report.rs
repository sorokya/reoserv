use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::talk::Report, AdminLevel},
};

use crate::{player::PlayerHandle, world::WorldHandle};

use super::handle_command::handle_command;

pub async fn report(
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut report = Report::default();
    report.deserialize(&reader);

    debug!("Recv: {:?}", report);

    if let Ok(character) = player.get_character().await {
        if report.message.starts_with('$') && character.admin_level != AdminLevel::Player {
            let args: Vec<&str> = report.message[1..].split_whitespace().collect();
            handle_command(args.as_slice(), &character, player, world).await;
        } else if let Ok(map) = player.get_map().await {
            let player_id = player.get_player_id().await?;
            map.send_chat_message(player_id, report.message);
        }
    }

    Ok(())
}
