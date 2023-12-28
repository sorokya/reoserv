use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::PlayersAcceptClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle, world::WorldHandle};

pub fn accept(reader: EoReader, player_id: i32, map: MapHandle) {
    let accept = match PlayersAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing PlayersAcceptClientPacket {}", e);
            return;
        }
    };

    map.find_player(player_id, accept.name);
}

pub fn list(player_id: i32, world: WorldHandle) {
    world.request_player_name_list(player_id);
}

pub fn request(player_id: i32, world: WorldHandle) {
    world.request_player_list(player_id);
}

pub async fn players(
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

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map {}", e);
            return;
        }
    };

    match action {
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::List => list(player_id, world),
        PacketAction::Request => request(player_id, world),
        _ => error!("Unhandled packet Players_{:?}", action),
    }
}
