use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::face, Direction, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub fn face(&mut self, target_player_id: EOShort, direction: Direction) {
        {
            let mut target = self.characters.get_mut(&target_player_id).unwrap();
            target.direction = direction;
        }

        let packet = face::Player {
            player_id: target_player_id,
            direction,
        };
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        let buf = builder.get();
        let target = self.characters.get(&target_player_id).unwrap();
        for character in self.characters.values() {
            if target_player_id != character.player_id.unwrap()
                && target.is_in_range(character.coords)
            {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    PacketAction::Player,
                    PacketFamily::Face,
                    buf.clone(),
                );
            }
        }
    }
}
