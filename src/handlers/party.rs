use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{PartyAcceptClientPacket, PartyRemoveClientPacket, PartyRequestClientPacket},
        PacketAction, PartyRequestType,
    },
};

use crate::{
    map::MapHandle,
    player::{PartyRequest, PlayerHandle},
    world::WorldHandle,
};

pub fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match PartyRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing PartyRequestClientPacket {}", e);
            return;
        }
    };

    match request.request_type {
        PartyRequestType::Join => {
            map.party_request(request.player_id, PartyRequest::Join(player_id))
        }
        PartyRequestType::Invite => {
            map.party_request(request.player_id, PartyRequest::Invite(player_id))
        }
        _ => {}
    }
}

pub fn accept(reader: EoReader, player_id: i32, world: WorldHandle) {
    let accept = match PartyAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing PartyAcceptClientPacket {}", e);
            return;
        }
    };

    world.accept_party_request(player_id, accept.inviter_player_id, accept.request_type);
}

pub fn remove(reader: EoReader, player_id: i32, world: WorldHandle) {
    let remove = match PartyRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing PartyRemoveClientPacket {}", e);
            return;
        }
    };

    world.remove_party_member(player_id, remove.player_id);
}

pub fn take(player_id: i32, world: WorldHandle) {
    world.request_party_list(player_id);
}

pub async fn party(
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
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::Accept => accept(reader, player_id, world),
        PacketAction::Remove => remove(reader, player_id, world),
        PacketAction::Take => take(player_id, world),
        _ => error!("Unhandled packet Party_{:?}", action),
    }
}
