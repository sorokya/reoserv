use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AccountAgreeClientPacket, AccountCreateClientPacket, AccountRequestClientPacket},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

fn create(reader: EoReader, player: PlayerHandle) {
    let create = match AccountCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing AccountCreateClientPacket {}", e);
            return;
        }
    };

    player.create_account(create);
}

fn request(reader: EoReader, player: PlayerHandle) {
    let request = match AccountRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing AccountRequestClientPacket {}", e);
            return;
        }
    };

    player.request_account_creation(request.username);
}

fn agree(reader: EoReader, player: PlayerHandle) {
    let agree = match AccountAgreeClientPacket::deserialize(&reader) {
        Ok(agree) => agree,
        Err(e) => {
            error!("Error deserializing AccountAgreeClientPacket {}", e);
            return;
        }
    };

    player.change_password(agree.username, agree.old_password, agree.new_password);
}

pub async fn account(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Create => create(reader, player),
        PacketAction::Request => request(reader, player),
        PacketAction::Agree => agree(reader, player),
        _ => error!("Unhandled packet Account_{:?}", action),
    }
}
