use eo::protocol::{server::players, PacketAction, PacketFamily, WarpAnimation};
use tokio::sync::oneshot;

use crate::character::Character;

use super::super::Map;

impl Map {
    pub fn enter(
        &mut self,
        new_character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    ) {
        if !new_character.hidden {
            let mut character_map_info = new_character.to_map_info();
            character_map_info.animation = warp_animation;

            let mut packet = players::Agree::default();
            packet.nearby.num_characters = 1;
            packet.nearby.characters.push(character_map_info);

            self.send_packet_near(
                &new_character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                packet,
            );
        }

        self.characters
            .insert(new_character.player_id.unwrap(), *new_character);

        let _ = respond_to.send(());
    }
}
