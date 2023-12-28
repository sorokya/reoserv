use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            WelcomeAgreeClientPacket, WelcomeAgreeClientPacketFileTypeData, WelcomeMsgClientPacket,
            WelcomeRequestClientPacket,
        },
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn agree(reader: EoReader, player_id: i32, world: WorldHandle) {
    let agree = match WelcomeAgreeClientPacket::deserialize(&reader) {
        Ok(agree) => agree,
        Err(e) => {
            error!("Error deserializing WelcomeAgreeClientPacket {}", e);
            return;
        }
    };

    world.get_file(
        player_id,
        agree.file_type,
        agree.session_id,
        match agree.file_type_data {
            Some(WelcomeAgreeClientPacketFileTypeData::Emf(_)) => None,
            Some(WelcomeAgreeClientPacketFileTypeData::Eif(agree_item)) => Some(agree_item.file_id),
            Some(WelcomeAgreeClientPacketFileTypeData::Enf(agree_npc)) => Some(agree_npc.file_id),
            Some(WelcomeAgreeClientPacketFileTypeData::Esf(agree_spell)) => Some(agree_spell.file_id),
            Some(WelcomeAgreeClientPacketFileTypeData::Ecf(agree_class)) => Some(agree_class.file_id),
            _ => return,
        },
        false,
    );
}

fn msg(reader: EoReader, player_id: i32, world: WorldHandle) {
    let msg = match WelcomeMsgClientPacket::deserialize(&reader) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error deserializing WelcomeMsgClientPacket {}", e);
            return;
        }
    };

    world.enter_game(player_id, msg.session_id);
}

fn request(reader: EoReader, player_id: i32, world: WorldHandle) {
    let request = match WelcomeRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing WelcomeRequestClientPacket {}", e);
            return;
        }
    };

    world.select_character(player_id, request.character_id);
}

pub async fn welcome(
    action: PacketAction,
    reader: EoReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    match action {
        PacketAction::Agree => agree(reader, player_id, world),
        PacketAction::Msg => msg(reader, player_id, world),
        PacketAction::Request => request(reader, player_id, world),
        _ => error!("Unhandled packet Welcome_{:?}", action),
    }
}
