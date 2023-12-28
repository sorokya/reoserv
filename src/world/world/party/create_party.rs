use eolib::{protocol::net::{PacketAction, PacketFamily}, data::EoWriter};

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

        let mut writer = EoWriter::new();
        writer.add_short(leader_id);
        writer.add_char(1);
        writer.add_char(leader_character.level);
        writer.add_char(leader_character.get_hp_percentage());
        writer.add_string(&leader_character.name);
        writer.add_byte(0xff);
        writer.add_short(member_id);
        writer.add_char(0);
        writer.add_char(member_character.level);
        writer.add_char(member_character.get_hp_percentage());
        writer.add_string(&member_character.name);

        let buf = writer.to_byte_array();

        leader.send(PacketAction::Create, PacketFamily::Party, buf.clone());
        member.send(PacketAction::Create, PacketFamily::Party, buf);
    }
}
