use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AdminInteractReportClientPacket, AdminInteractTellClientPacket},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn report(reader: EoReader, player_id: i32, world: WorldHandle) {
    let report = match AdminInteractReportClientPacket::deserialize(&reader) {
        Ok(report) => report,
        Err(e) => {
            error!("Error deserializing AdminInteractReportClientPacket {}", e);
            return;
        }
    };

    world.report_player(player_id, report.reportee, report.message);
}

fn tell(reader: EoReader, player_id: i32, world: WorldHandle) {
    let tell = match AdminInteractTellClientPacket::deserialize(&reader) {
        Ok(tell) => tell,
        Err(e) => {
            error!("Error deserializing AdminInteractTellClientPacket {}", e);
            return;
        }
    };

    world.send_admin_message(player_id, tell.message);
}

pub async fn admin_interact(
    action: PacketAction,
    reader: EoReader,
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
