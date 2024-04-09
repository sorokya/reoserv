use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            BoardCreateClientPacket, BoardOpenClientPacket, BoardRemoveClientPacket,
            BoardTakeClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn board_create(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let create = match BoardCreateClientPacket::deserialize(&reader) {
                Ok(create) => create,
                Err(e) => {
                    error!("Error deserializing BoardCreateClientPacket {}", e);
                    return;
                }
            };

            map.create_board_post(self.id, create.post_subject, create.post_body);
        }
    }

    fn board_open(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let open = match BoardOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing BoardOpenClientPacket {}", e);
                    return;
                }
            };
            map.open_board(self.id, open.board_id + 1);
        }
    }

    fn board_remove(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let remove = match BoardRemoveClientPacket::deserialize(&reader) {
                Ok(remove) => remove,
                Err(e) => {
                    error!("Error deserializing BoardOpenClientPacket {}", e);
                    return;
                }
            };
            map.remove_board_post(self.id, remove.post_id);
        }
    }

    fn board_take(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let take = match BoardTakeClientPacket::deserialize(&reader) {
                Ok(take) => take,
                Err(e) => {
                    error!("Error deserializing BoardTakeClientPacket {}", e);
                    return;
                }
            };
            map.view_board_post(self.id, take.post_id);
        }
    }

    pub fn handle_board(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Create => self.board_create(reader),
            PacketAction::Open => self.board_open(reader),
            PacketAction::Remove => self.board_remove(reader),
            PacketAction::Take => self.board_take(reader),
            _ => error!("Unhandled packet Board_{:?}", action),
        }
    }
}
