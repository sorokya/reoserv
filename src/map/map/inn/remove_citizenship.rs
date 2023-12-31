use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{CitizenRemoveServerPacket, InnUnsubscribeReply},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{INN_DB, NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn remove_citizenship(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let npc_index = match player.get_interact_npc_index().await {
            Some(npc_index) => npc_index,
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

        if npc_data.r#type != NpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.behavior_id == npc_data.behavior_id)
        {
            Some(inn_data) => inn_data,
            None => return,
        };

        let packet = CitizenRemoveServerPacket {
            reply_code: if character.home == SETTINGS.new_character.home
                || character.home != inn_data.name
            {
                InnUnsubscribeReply::NotCitizen
            } else {
                character.home = SETTINGS.new_character.home.to_owned();
                InnUnsubscribeReply::Unsubscribed
            },
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize CitizenRemoveServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Remove,
            PacketFamily::Citizen,
            writer.to_byte_array(),
        );
    }
}
