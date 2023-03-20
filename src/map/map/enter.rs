use eo::{
    data::Serializeable,
    protocol::{server::players, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::character::Character;

use super::Map;

impl Map {
    pub fn enter(
        &mut self,
        new_character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    ) {
        let mut character_map_info = new_character.to_map_info();
        character_map_info.animation = warp_animation;

        // TODO: Look into queueing this packet? (e.g multiple people entering the map at once)
        let mut packet = players::Agree::default();
        packet.nearby.num_characters = 1;
        packet.nearby.characters.push(character_map_info);
        let buf = packet.serialize();
        for character in self.characters.values() {
            if new_character.is_in_range(character.coords) {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    PacketAction::Agree,
                    PacketFamily::Players,
                    buf.clone(),
                );
            }
        }
        self.characters
            .insert(new_character.player_id.unwrap(), *new_character);
        let _ = respond_to.send(());
    }
}
