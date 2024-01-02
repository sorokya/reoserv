use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{
                AvatarAgreeServerPacket, AvatarChange, AvatarChangeChangeTypeData,
                AvatarChangeChangeTypeDataHair, AvatarChangeType, BarberAgreeServerPacket,
            },
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn buy_haircut(
        &mut self,
        player_id: i32,
        session_id: i32,
        hair_style: i32,
        hair_color: i32,
    ) {
        if hair_style < 0
            || hair_style > SETTINGS.character.max_hair_style
            || hair_color < 0
            || hair_color > SETTINGS.character.max_hair_color
        {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let actual_session_id = match character.player.as_ref().unwrap().get_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get session id {}", e);
                return;
            }
        };

        if actual_session_id != session_id {
            return;
        }

        let npc_index = match character
            .player
            .as_ref()
            .unwrap()
            .get_interact_npc_index()
            .await
        {
            Some(index) => index,
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

        let cost = cmp::max(1, character.level) * SETTINGS.barber.cost_per_level;

        if character.get_item_amount(1) < cost {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, cost);
        character.hair_style = hair_style;
        character.hair_color = hair_color;

        let change = AvatarChange {
            player_id,
            sound: false,
            change_type: AvatarChangeType::Hair,
            change_type_data: Some(AvatarChangeChangeTypeData::Hair(
                AvatarChangeChangeTypeDataHair {
                    hair_style,
                    hair_color,
                },
            )),
        };

        let packet = BarberAgreeServerPacket {
            gold_amount: character.get_item_amount(1),
            change: change.clone(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize BarberAgreeServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Agree,
            PacketFamily::Barber,
            writer.to_byte_array(),
        );

        let packet = AvatarAgreeServerPacket { change };

        self.send_packet_near_player(player_id, PacketAction::Agree, PacketFamily::Avatar, packet);
    }
}
