use eolib::{
    data::EoWriter,
    protocol::net::{PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn refresh_party(&self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let party = match self.get_player_party(player_id) {
            Some(party) => party,
            None => return,
        };

        let mut writer = EoWriter::new();
        let leader_id = party.leader;
        for (index, member_id) in party.members.iter().enumerate() {
            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            let character = match member.get_character().await {
                Ok(character) => character,
                Err(_) => continue,
            };

            writer.add_short(*member_id);
            writer.add_char(if *member_id == leader_id { 1 } else { 0 });
            writer.add_char(character.level);
            writer.add_char(character.get_hp_percentage());
            writer.add_string(&character.name);
            if index != party.members.len() - 1 {
                writer.add_byte(0xff);
            }
        }

        player.send(
            PacketAction::List,
            PacketFamily::Party,
            writer.to_byte_array(),
        );
    }
}
