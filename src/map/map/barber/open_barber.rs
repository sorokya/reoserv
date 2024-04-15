use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::BarberOpenServerPacket, PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::{utils::in_client_range, NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn open_barber(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
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
            error!("Error serializing BarberOpenServerPacket: {}", e);
            return;
        }

        if character.is_deep {
            writer.add_short(SETTINGS.character.max_hair_style).unwrap();
            writer.add_short(SETTINGS.barber.base_cost).unwrap();
            writer.add_short(SETTINGS.barber.cost_per_level).unwrap();
        }

        player.send_buf(
            PacketAction::Open,
            PacketFamily::Barber,
            writer.to_byte_array(),
        );
    }
}
