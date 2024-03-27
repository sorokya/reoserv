use std::cmp;

use eolib::protocol::net::{server::RecoverReplyServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn give_karma(&mut self, player_id: i32, amount: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let amount = cmp::min(2000 - character.karma, amount);

        character.karma += amount;

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        player.send(
            PacketAction::Reply,
            PacketFamily::Recover,
            &RecoverReplyServerPacket {
                experience: character.experience,
                karma: character.karma,
                ..Default::default()
            },
        );
    }
}
