use eolib::{
    data::EoWriter,
    protocol::net::{PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn update_party_hp(&self, player_id: i32, hp_percentage: i32) {
        if let Some(party) = self.get_player_party(player_id) {
            let mut writer = EoWriter::new();
            writer.add_short(player_id);
            writer.add_char(hp_percentage);

            let buf = writer.to_byte_array();

            for member_id in &party.members {
                let member = match self.players.get(member_id) {
                    Some(member) => member,
                    None => continue,
                };

                member.send(PacketAction::Agree, PacketFamily::Party, buf.clone());
            }
        }
    }
}
