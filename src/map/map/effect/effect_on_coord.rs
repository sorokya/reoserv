use eolib::protocol::{
    net::{server::EffectAgreeServerPacket, PacketAction, PacketFamily},
    Coords,
};

use super::super::Map;

impl Map {
    pub fn effect_on_coord(&self, coords: Coords, effect_id: i32) {
        self.send_packet_near(
            &coords,
            PacketAction::Agree,
            PacketFamily::Effect,
            EffectAgreeServerPacket { coords, effect_id },
        );
    }
}
