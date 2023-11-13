use eo::{
    data::{EOChar, EOShort, StreamReader},
    protocol::PacketAction,
};

use crate::{
    map::MapHandle,
    player::{PartyRequest, PlayerHandle},
};

const JOIN: EOChar = 0;
const INVITE: EOChar = 1;

pub fn request(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let request_type = reader.get_char();
    let target_player_id = reader.get_short();

    match request_type {
        JOIN => map.party_request(target_player_id, PartyRequest::Join(player_id)),
        INVITE => map.party_request(target_player_id, PartyRequest::Invite(player_id)),
        _ => {}
    }
}

pub fn accept(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let request_type = reader.get_char();
    let target_player_id = reader.get_short();
    map.accept_party_request(player_id, target_player_id, request_type);
}

pub fn remove(reader: StreamReader, player_id: EOShort, map: MapHandle) {}

pub fn take(reader: StreamReader, player_id: EOShort, map: MapHandle) {}

pub async fn party(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::Remove => remove(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Party_{:?}", action),
    }
}
