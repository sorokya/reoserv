use eo::{
    data::EOShort,
    protocol::{server::avatar, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::character::Character;

use super::super::Map;

impl Map {
    pub fn leave(
        &mut self,
        player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
        interact_player_id: Option<EOShort>,
    ) {
        if let Some(interact_player_id) = interact_player_id {
            self.cancel_trade(player_id, interact_player_id);
        }

        let target = self.characters.remove(&player_id).unwrap();
        if !target.hidden {
            let packet = avatar::Remove {
                player_id,
                animation: warp_animation,
            };

            self.send_packet_near(
                &target.coords,
                PacketAction::Remove,
                PacketFamily::Avatar,
                packet,
            );
        }

        let _ = respond_to.send(target);
    }
}
