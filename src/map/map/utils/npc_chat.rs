use eolib::protocol::net::{PacketAction, PacketFamily, server::NpcDialogServerPacket};

use super::super::Map;

impl Map {
    pub fn npc_chat(&self, npc_index: i32, message: &str) {
        let npc = match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => npc,
            None => return,
        };

        self.send_packet_near(
            &npc.coords,
            PacketAction::Dialog,
            PacketFamily::Npc,
            NpcDialogServerPacket {
                npc_index,
                message: message.to_owned(),
            },
        );
    }
}
