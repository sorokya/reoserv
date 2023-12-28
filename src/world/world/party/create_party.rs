use eo::{
    data::{i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::world::Party;

use super::super::World;

impl World {
    pub async fn create_party(&mut self, leader_id: i32, member_id: i32) {
        let leader = match self.players.get(&leader_id) {
            Some(player) => player,
            None => return,
        };

        let leader_character = match leader.get_character().await {
            Ok(character) => character,
            Err(_) => return,
        };

        let member = match self.players.get(&member_id) {
            Some(player) => player,
            None => return,
        };

        let member_character = match member.get_character().await {
            Ok(character) => character,
            Err(_) => return,
        };

        self.parties.push(Party::new(leader_id, member_id));

        let mut builder = StreamBuilder::new();
        builder.add_short(leader_id);
        builder.add_char(1);
        builder.add_char(leader_character.level);
        builder.add_char(leader_character.get_hp_percentage());
        builder.add_break_string(&leader_character.name);
        builder.add_short(member_id);
        builder.add_char(0);
        builder.add_char(member_character.level);
        builder.add_char(member_character.get_hp_percentage());
        builder.add_string(&member_character.name);

        let buf = builder.get();

        leader.send(PacketAction::Create, PacketFamily::Party, buf.clone());
        member.send(PacketAction::Create, PacketFamily::Party, buf);
    }
}
