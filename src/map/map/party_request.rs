use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::{player::PartyRequest, utils::in_client_range};

use super::Map;

impl Map {
    pub async fn party_request(&self, target_player_id: EOShort, request: PartyRequest) {
        let player_id = match request {
            PartyRequest::Join(id) => id,
            PartyRequest::Invite(id) => id,
            _ => return,
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if !in_client_range(&character.coords, &target_character.coords) {
            return;
        }

        let target = match target_character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        target.set_party_request(request);

        let mut builder = StreamBuilder::new();
        builder.add_char(match request {
            PartyRequest::Invite(_) => 1,
            PartyRequest::Join(_) => 0,
            _ => return,
        });
        builder.add_short(player_id);
        builder.add_string(&character.name);

        target.send(PacketAction::Request, PacketFamily::Party, builder.get());
    }
}
