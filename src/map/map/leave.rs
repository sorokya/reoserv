use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
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
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        let buf = builder.get();
        for character in self.characters.values() {
            if target.is_in_range(&character.coords) {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    PacketAction::Remove,
                    PacketFamily::Avatar,
                    buf.clone(),
                );
            }
        }
        let _ = respond_to.send(target);
    }
}
