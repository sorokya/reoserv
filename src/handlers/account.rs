use eo::{
    data::{i32, Serializeable, StreamReader},
    protocol::{
        client::account::{Create, Request},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn create(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut create = Create::default();
    create.deserialize(&reader);
    world.create_account(player_id, create.clone());
}

fn request(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);
    world.request_account_creation(player_id, request.username);
}

fn agree(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let username = reader.get_break_string();
    let current_password = reader.get_break_string();
    let new_password = reader.get_break_string();

    world.change_password(player_id, username, current_password, new_password);
}

pub async fn account(
    action: PacketAction,
    reader: StreamReader,
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
