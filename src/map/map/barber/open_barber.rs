use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{PacketAction, PacketFamily, server::BarberOpenServerPacket},
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SETTINGS, utils::in_client_range};

use super::super::Map;

impl Map {
    pub fn open_barber(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_db = NPC_DB.load();
        let npc_data = match npc_db.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Barber {
            return;
        }

        if !in_client_range(&character.coords, &npc.coords) {
            return;
        }

        let player = match &character.player {
            Some(player) => player,
            None => return,
        };

        player.set_interact_npc_index(npc_index);

        let packet = BarberOpenServerPacket { session_id };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            tracing::error!("Error serializing BarberOpenServerPacket: {}", e);
            return;
        }

        if character.is_deep {
            writer
                .add_short(SETTINGS.load().character.max_hair_style)
                .unwrap();
            writer.add_short(SETTINGS.load().barber.base_cost).unwrap();
            writer
                .add_short(SETTINGS.load().barber.cost_per_level)
                .unwrap();
        }

        player.send_buf(
            PacketAction::Open,
            PacketFamily::Barber,
            writer.to_byte_array(),
        );
    }
}
