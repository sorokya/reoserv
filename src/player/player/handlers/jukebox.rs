use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::JukeboxMsgClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn jukebox_open(&mut self) {
        if let Some(map) = &self.map {
            map.open_jukebox(self.id);
        }
    }

    fn jukebox_msg(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let msg = match JukeboxMsgClientPacket::deserialize(&reader) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error deserializing JukeboxMsgClientPacket {}", e);
                    return;
                }
            };

            map.play_jukebox_track(self.id, msg.track_id + 1);
        }
    }

    pub fn handle_jukebox(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Open => self.jukebox_open(),
            PacketAction::Msg => self.jukebox_msg(reader),
            _ => error!("Unhandled packet Jukebox_{:?}", action),
        }
    }
}
