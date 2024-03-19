use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{PartyAddServerPacket, PartyCreateServerPacket, PartyMember},
        PacketAction, PacketFamily,
    },
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

        let packet = PartyAddServerPacket {
            member: PartyMember {
                player_id,
                leader: false,
                level: character.level,
                hp_percentage: character.get_hp_percentage(),
                name: character.name.clone(),
            },
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing PartyAddServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        let party = match self
            .parties
            .iter()
            .find(|p| p.members.contains(&party_member_id))
        {
            Some(party) => party,
            None => return,
        };

        for member_id in &party.members {
            if *member_id == player_id {
                continue;
            }

            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send_buf(PacketAction::Add, PacketFamily::Party, buf.clone());
        }

        player.send(
            PacketAction::Create,
            PacketFamily::Party,
            &PartyCreateServerPacket {
                members: self.get_party_members(party).await,
            },
        );
    }
}
