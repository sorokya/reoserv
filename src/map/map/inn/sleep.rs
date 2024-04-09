use eolib::protocol::{
    net::{server::CitizenAcceptServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
    Coords,
};

use crate::{INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub fn sleep(&mut self, player_id: i32, npc_index: i32, cost: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
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

        if npc_data.r#type != NpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.behavior_id == npc_data.behavior_id)
        {
            Some(inn) => inn,
            None => return,
        };

        if inn_data.name != character.home {
            return;
        }

        if cost == 0 {
            return;
        }

        if character.get_item_amount(1) < cost {
            return;
        }

        character.remove_item(1, cost);
        character.hp = character.max_hp;
        character.tp = character.max_tp;

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Accept,
                PacketFamily::Citizen,
                &CitizenAcceptServerPacket {
                    gold_amount: character.get_item_amount(1),
                },
            );

            player.request_warp(
                inn_data.sleep_map,
                Coords {
                    x: inn_data.sleep_x,
                    y: inn_data.sleep_y,
                },
                inn_data.sleep_map == self.id,
                None,
            );

            player.update_party_hp(100);
        }
    }
}
