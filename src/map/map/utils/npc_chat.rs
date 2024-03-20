use eolib::protocol::net::{server::NpcDialogServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn npc_chat(&self, npc_index: i32, message: &str) {
        let npc = match self.npcs.get(&npc_index) {
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
