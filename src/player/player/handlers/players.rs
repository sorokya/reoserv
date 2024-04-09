use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::PlayersAcceptClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn players_accept(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let accept = match PlayersAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing PlayersAcceptClientPacket {}", e);
                    return;
                }
            };

            map.find_player(self.id, accept.name);
        }
    }

    fn players_list(&mut self) {
        self.world.request_player_name_list(self.id);
    }

    fn players_request(&mut self) {
        self.world.request_player_list(self.id);
    }

    pub fn handle_players(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Accept => self.players_accept(reader),
            PacketAction::List => self.players_list(),
            PacketAction::Request => self.players_request(),
            _ => error!("Unhandled packet Players_{:?}", action),
        }
    }
}
