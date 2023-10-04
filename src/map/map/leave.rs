use eo::{
    data::EOShort,
    protocol::{server::avatar, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::character::Character;

use super::Map;

impl Map {
    pub fn leave(
        &mut self,
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
    ) {
        let target = self.characters.remove(&target_player_id).unwrap();
        let packet = avatar::Remove {
            player_id: target_player_id,
            animation: warp_animation,
        };

        self.send_packet_near(
            &target.coords,
            PacketAction::Remove,
            PacketFamily::Avatar,
            packet,
        );

        let _ = respond_to.send(target);
    }
}
