use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AccountAgreeClientPacket, AccountCreateClientPacket, AccountRequestClientPacket},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn create(reader: EoReader, player_id: i32, world: WorldHandle) {
    let create = match AccountCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing AccountCreateClientPacket {}", e);
            return;
        }
    };

    world.create_account(player_id, create.clone());
}

fn request(reader: EoReader, player_id: i32, world: WorldHandle) {
    let request = match AccountRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing AccountRequestClientPacket {}", e);
            return;
        }
    };

    world.request_account_creation(player_id, request.username);
}

fn agree(reader: EoReader, player_id: i32, world: WorldHandle) {
    let agree = match AccountAgreeClientPacket::deserialize(&reader) {
        Ok(agree) => agree,
        Err(e) => {
            error!("Error deserializing AccountAgreeClientPacket {}", e);
            return;
        }
    };

    world.change_password(
        player_id,
        agree.username,
        agree.old_password,
        agree.new_password,
    );
}

pub async fn account(
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
        PacketAction::Create => create(reader, player_id, world),
        PacketAction::Request => request(reader, player_id, world),
        PacketAction::Agree => agree(reader, player_id, world),
        _ => error!("Unhandled packet Account_{:?}", action),
    }
}
