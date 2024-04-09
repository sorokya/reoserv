use eolib::protocol::net::PacketAction;

use super::super::Player;

impl Player {
    fn refresh_request(&mut self) {
        if let Some(map) = &self.map {
            map.request_refresh(self.id);
        }
    }

    pub fn handle_refresh(&mut self, action: PacketAction) {
        match action {
            PacketAction::Request => self.refresh_request(),
            _ => error!("Unhandled packet Refresh_{:?}", action),
        }
    }
}
