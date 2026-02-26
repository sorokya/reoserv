use eolib::protocol::net::{
    PacketAction, PacketFamily,
    server::{NearbyInfo, PlayersAgreeServerPacket, WarpEffect},
};
use tokio::sync::oneshot;

use crate::{
    NPC_DB,
    character::Character,
    deep::{BossPingServerPacket, FAMILY_BOSS},
    utils::in_client_range,
};

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

        if character.is_deep
            && let Some(player) = &character.player
        {
            for npc in self.npcs.iter().filter(|npc| {
                let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                    Some(npc) => npc,
                    None => return false,
                };

                npc.alive && npc_data.boss && in_client_range(&character.coords, &npc.coords)
            }) {
                player.send(
                    PacketAction::Ping,
                    PacketFamily::Unrecognized(FAMILY_BOSS),
                    &BossPingServerPacket {
                        npc_index: npc.index,
                        npc_id: npc.id,
                        hp: npc.hp,
                        hp_percentage: npc.get_hp_percentage(),
                        killed: false,
                    },
                );
            }
        }

        self.characters
            .insert(character.player_id.unwrap(), character);

        let _ = respond_to.send(());
    }
}
