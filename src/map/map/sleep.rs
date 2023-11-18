use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::{INN_DB, NPC_DB};

use super::Map;

impl Map {
    pub async fn sleep(&mut self, player_id: EOShort, session_id: EOShort) {
        let (cost, sleep_map, sleep_coords) = {
            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            let player = match character.player.as_ref() {
                Some(player) => player,
                None => return,
            };

            let actual_session_id = match player.get_session_id().await {
                Ok(session_id) => session_id,
                Err(_) => return,
            };

            if session_id != actual_session_id {
                return;
            }

            let npc_index = match player.get_interact_npc_index().await {
                Some(npc_index) => npc_index,
                None => return,
            };

            let npc = match self.npcs.get(&npc_index) {
                Some(npc) => npc,
                None => return,
            };

            let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != EnfNpcType::Inn {
                return;
            }

            let inn_data = match INN_DB
                .inns
                .iter()
                .find(|inn| inn.vendor_id == npc_data.behavior_id)
            {
                Some(inn) => inn,
                None => return,
            };

            if inn_data.name != character.home {
                return;
            }

            let cost = match player.get_sleep_cost().await {
                Some(cost) => cost,
                None => return,
            };

            if cost == 0 {
                return;
            }

            if character.get_item_amount(1) < cost {
                return;
            }

            (
                cost,
                inn_data.sleep_map,
                Coords {
                    x: inn_data.sleep_x,
                    y: inn_data.sleep_y,
                },
            )
        };

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, cost);
        character.hp = character.max_hp;
        character.tp = character.max_tp;

        let mut builder = StreamBuilder::new();
        builder.add_int(character.get_item_amount(1));

        character.player.as_ref().unwrap().send(
            PacketAction::Accept,
            PacketFamily::Citizen,
            builder.get(),
        );

        character.player.as_ref().unwrap().request_warp(
            sleep_map,
            sleep_coords,
            sleep_map == self.id,
            None,
        );

        character.player.as_ref().unwrap().update_party_hp(100);
    }
}
