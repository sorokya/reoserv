use bytes::Bytes;
use eolib::{protocol::net::{PacketAction, PacketFamily}, data::EoWriter};

use super::super::World;

impl World {
    pub fn leave_party(&mut self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let party = match self
            .parties
            .iter_mut()
            .find(|p| p.members.contains(&player_id))
        {
            Some(party) => party,
            None => return,
        };

        party.members.retain(|&id| id != player_id);

        player.send(
            PacketAction::Close,
            PacketFamily::Party,
            Bytes::from_static(&[0xff]),
        );

        let mut writer = EoWriter::new();
        writer.add_short(player_id);

        let buf = writer.to_byte_array();

        for member_id in &party.members {
            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send(PacketAction::Remove, PacketFamily::Party, buf.clone());
        }
    }
}
