use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{PartyCreateServerPacket, PartyMember},
        PacketAction, PacketFamily,
    },
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

        let packet = PartyCreateServerPacket {
            members: vec![
                PartyMember {
                    player_id: leader_id,
                    leader: true,
                    level: leader_character.level,
                    hp_percentage: leader_character.get_hp_percentage(),
                    name: leader_character.name.clone(),
                },
                PartyMember {
                    player_id: member_id,
                    leader: false,
                    level: member_character.level,
                    hp_percentage: member_character.get_hp_percentage(),
                    name: member_character.name.clone(),
                },
            ],
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing PartyCreateServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        leader.send(PacketAction::Create, PacketFamily::Party, buf.clone());
        member.send(PacketAction::Create, PacketFamily::Party, buf);
    }
}
