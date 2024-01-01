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

use crate::player::PlayerHandle;

fn agree(reader: EoReader, player: PlayerHandle) {
    let agree = match WelcomeAgreeClientPacket::deserialize(&reader) {
        Ok(agree) => agree,
        Err(e) => {
            error!("Error deserializing WelcomeAgreeClientPacket {}", e);
            return;
        }
    };

    player.get_file(
        agree.file_type,
        agree.session_id,
        match agree.file_type_data {
            Some(WelcomeAgreeClientPacketFileTypeData::Emf(_)) => None,
            Some(WelcomeAgreeClientPacketFileTypeData::Eif(agree_item)) => Some(agree_item.file_id),
            Some(WelcomeAgreeClientPacketFileTypeData::Enf(agree_npc)) => Some(agree_npc.file_id),
            Some(WelcomeAgreeClientPacketFileTypeData::Esf(agree_spell)) => {
                Some(agree_spell.file_id)
            }
            Some(WelcomeAgreeClientPacketFileTypeData::Ecf(agree_class)) => {
                Some(agree_class.file_id)
            }
            _ => return,
        },
        false,
    );
}

fn msg(reader: EoReader, player: PlayerHandle) {
    let msg = match WelcomeMsgClientPacket::deserialize(&reader) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error deserializing WelcomeMsgClientPacket {}", e);
            return;
        }
    };

    player.enter_game(msg.session_id);
}

fn request(reader: EoReader, player: PlayerHandle) {
    let request = match WelcomeRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing WelcomeRequestClientPacket {}", e);
            return;
        }
    };

    player.select_character(request.character_id);
}

pub async fn welcome(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Agree => agree(reader, player),
        PacketAction::Msg => msg(reader, player),
        PacketAction::Request => request(reader, player),
        _ => error!("Unhandled packet Welcome_{:?}", action),
    }
}
