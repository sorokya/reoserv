use eo::{
    data::i32,
    protocol::{server::avatar, PacketAction, PacketFamily, WarpAnimation},
};
use tokio::sync::oneshot;

use crate::{character::Character, ARENAS};

use super::super::Map;

impl Map {
    pub fn leave(
        &mut self,
        player_id: i32,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
        interact_player_id: Option<i32>,
    ) {
        if let Some(interact_player_id) = interact_player_id {
            self.cancel_trade(player_id, interact_player_id);
        }

        let target = self.characters.remove(&player_id).unwrap();

        if let Some(config) = ARENAS.arenas.iter().find(|a| a.map == self.id) {
            if self.arena_players.iter().any(|p| p.player_id == player_id)
                && !config
                    .spawns
                    .iter()
                    .any(|s| s.from.x == target.coords.x && s.from.y == target.coords.y)
            {
                self.arena_players.retain(|a| a.player_id != player_id);
                if self.arena_players.len() == 1 {
                    self.abandon_arena();
                }
            }
        }

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
