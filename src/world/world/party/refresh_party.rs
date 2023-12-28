use eo::{
    data::{i32, StreamBuilder, EO_BREAK_CHAR},
    protocol::{PacketAction, PacketFamily},
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

        let mut builder = StreamBuilder::new();
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

            builder.add_short(*member_id);
            builder.add_char(if *member_id == leader_id { 1 } else { 0 });
            builder.add_char(character.level);
            builder.add_char(character.get_hp_percentage());
            builder.add_string(&character.name);
            if index != party.members.len() - 1 {
                builder.add_byte(EO_BREAK_CHAR);
            }
        }

        player.send(PacketAction::List, PacketFamily::Party, builder.get());
    }
}
