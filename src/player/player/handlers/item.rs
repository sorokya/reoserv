use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            ItemDropClientPacket, ItemGetClientPacket, ItemJunkClientPacket, ItemUseClientPacket,
        },
        PacketAction,
    },
};

use crate::{deep::ItemReportClientPacket, SETTINGS};

use super::super::Player;

impl Player {
    fn item_drop(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let drop = match ItemDropClientPacket::deserialize(&reader) {
                Ok(drop) => drop,
                Err(e) => {
                    error!("Error deserializing ItemDropClientPacket {}", e);
                    return;
                }
            };

            map.drop_item(self.id, drop.item, drop.coords);
        }
    }

    fn item_get(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let get = match ItemGetClientPacket::deserialize(&reader) {
                Ok(get) => get,
                Err(e) => {
                    error!("Error deserializing ItemGetClientPacket {}", e);
                    return;
                }
            };

            map.get_item(self.id, get.item_index);
        }
    }

    fn item_junk(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let junk = match ItemJunkClientPacket::deserialize(&reader) {
                Ok(junk) => junk,
                Err(e) => {
                    error!("Error deserializing ItemJunkClientPacket {}", e);
                    return;
                }
            };

            map.junk_item(self.id, junk.item.id, junk.item.amount);
        }
    }

    fn item_use(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let packet = match ItemUseClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing ItemUseClientPacket {}", e);
                    return;
                }
            };

            map.use_item(self.id, packet.item_id);
        }
    }

    fn item_report(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let packet = match ItemReportClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing ItemReportClientPacket: {}", e);
                    return;
                }
            };

            if packet.title.len() > SETTINGS.character.max_title_length {
                return;
            }

            map.use_title_item(self.id, packet.item_id, packet.title);
        }
    }

    pub fn handle_item(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with items when trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Drop => self.item_drop(reader),
            PacketAction::Get => self.item_get(reader),
            PacketAction::Junk => self.item_junk(reader),
            PacketAction::Use => self.item_use(reader),
            PacketAction::Report => self.item_report(reader),
            _ => error!("Unhandled packet Item_{:?}", action),
        }
    }
}
