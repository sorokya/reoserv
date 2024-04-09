use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::EmoteReportClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn emote_report(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let report = match EmoteReportClientPacket::deserialize(&reader) {
                Ok(report) => report,
                Err(e) => {
                    error!("Error deserializing EmoteReportClientPacket {}", e);
                    return;
                }
            };

            map.emote(self.id, report.emote);
        }
    }

    pub fn handle_emote(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Report => self.emote_report(reader),
            _ => error!("Unhandled packet Emote_{:?}", action),
        }
    }
}
