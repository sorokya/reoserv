use eo::{
    data::{i32, StreamBuilder, EO_BREAK_CHAR},
    protocol::{PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn join_party(&mut self, player_id: i32, party_member_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let party = match self
            .parties
            .iter_mut()
            .find(|p| p.members.contains(&party_member_id))
        {
            Some(party) => party,
            None => return,
        };

        party.members.push(player_id);

        let character = match player.get_character().await {
            Ok(character) => character,
            Err(_) => return,
        };

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_char(0);
        builder.add_char(character.level);
        builder.add_char(character.get_hp_percentage());
        builder.add_string(&character.name);

        let buf = builder.get();

        for member_id in &party.members {
            if *member_id == player_id {
                continue;
            }

            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send(PacketAction::Add, PacketFamily::Party, buf.clone());
        }

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

        player.send(PacketAction::Create, PacketFamily::Party, builder.get());
    }
}
