use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AdminInteractReportClientPacket, AdminInteractTellClientPacket},
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn admin_interact_report(&mut self, reader: EoReader) {
        let report = match AdminInteractReportClientPacket::deserialize(&reader) {
            Ok(report) => report,
            Err(e) => {
                error!("Error deserializing AdminInteractReportClientPacket {}", e);
                return;
            }
        };

        let world = self.world.clone();
        let player_id = self.id;

        tokio::spawn(async move {
            world.report_player(player_id, report.reportee, report.message);
        });
    }

    fn admin_interact_tell(&mut self, reader: EoReader) {
        let tell = match AdminInteractTellClientPacket::deserialize(&reader) {
            Ok(tell) => tell,
            Err(e) => {
                error!("Error deserializing AdminInteractTellClientPacket {}", e);
                return;
            }
        };

        let world = self.world.to_owned();
        let player_id = self.id;

        tokio::spawn(async move {
            world.send_admin_message(player_id, tell.message);
        });
    }

    pub fn handle_admin_interact(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Report => self.admin_interact_report(reader),
            PacketAction::Tell => self.admin_interact_tell(reader),
            _ => error!("Unhandled packet AdminInteract_{:?}", action),
        }
    }
}
