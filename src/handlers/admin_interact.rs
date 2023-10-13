use eo::{
    data::{EOShort, StreamReader},
    protocol::PacketAction,
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn report(reader: StreamReader, player_id: EOShort, world: WorldHandle) {
    let reportee_name = reader.get_break_string();
    let message = reader.get_break_string();
    world.report_player(player_id, reportee_name, message);
}

fn tell(reader: StreamReader, player_id: EOShort, world: WorldHandle) {
    let message = reader.get_break_string();
    world.send_admin_message(player_id, message);
}

pub async fn admin_interact(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Report => report(reader, player_id, world),
        PacketAction::Tell => tell(reader, player_id, world),
        _ => error!("Unhandled packet AdminInteract_{:?}", action),
    }
}
