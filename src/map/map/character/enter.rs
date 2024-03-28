use eolib::protocol::net::{
    server::{NearbyInfo, PlayersAgreeServerPacket, WarpEffect},
    PacketAction, PacketFamily,
};
use tokio::sync::oneshot;

use crate::character::Character;

use super::super::Map;

impl Map {
    pub fn enter(
        &mut self,
        new_character: Box<Character>,
        warp_animation: Option<WarpEffect>,
        respond_to: oneshot::Sender<()>,
    ) {
        if !new_character.hidden {
            let mut character_map_info = new_character.to_map_info();
            character_map_info.warp_effect = warp_animation;

            let packet = PlayersAgreeServerPacket {
                nearby: NearbyInfo {
                    characters: vec![character_map_info],
                    ..Default::default()
                },
            };

            self.send_packet_near(
                &new_character.coords,
                PacketAction::Agree,
                PacketFamily::Players,
                packet,
            );
        }

        let mut character = *new_character;

        character.entered_map();

        self.characters
            .insert(character.player_id.unwrap(), character);

        let _ = respond_to.send(());
    }
}
