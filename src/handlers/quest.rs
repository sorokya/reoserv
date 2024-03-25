use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            QuestAcceptClientPacket, QuestAcceptClientPacketReplyTypeData, QuestUseClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn r#use(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let r#use = match QuestUseClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing QuestUseClientPacket: {}", e);
            return;
        }
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Failed to generate session_id: {}", e);
            return;
        }
    };

    map.talk_to_quest_npc(player_id, r#use.npc_index, r#use.quest_id, session_id);
}

async fn accept(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let accept = match QuestAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing QuestAcceptClientPacket: {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != accept.session_id {
                return;
            }
        }
        Err(_) => {
            return;
        }
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => {
            return;
        }
    };

    let action_id = match accept.reply_type_data {
        Some(QuestAcceptClientPacketReplyTypeData::Link(data)) => Some(data.action),
        _ => None,
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Failed to generate session_id: {}", e);
            return;
        }
    };

    map.reply_to_quest_npc(player_id, npc_index, accept.quest_id, session_id, action_id);
}

pub async fn quest(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Use => r#use(reader, player, player_id, map).await,
        PacketAction::Accept => accept(reader, player, player_id, map).await,
        _ => error!("Unhandled packet Quest_{:?}", action),
    }
}
