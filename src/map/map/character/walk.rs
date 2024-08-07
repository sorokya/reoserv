use eolib::protocol::{
    map::MapTileSpec,
    net::{
        server::{WalkPlayerServerPacket, WalkReplyServerPacket},
        PacketAction, PacketFamily,
    },
    Coords, Direction,
};

use crate::{
    utils::{get_next_coords, in_client_range},
    SETTINGS,
};

use super::super::Map;

impl Map {
    pub fn walk(&mut self, player_id: i32, direction: Direction, client_coords: Coords) {
        if let Some((previous_coords, coords, player, hidden)) = {
            let (coords, admin_level, player, hidden, ghost_ticks) =
                match self.characters.get(&player_id) {
                    Some(character) => (
                        character.coords,
                        character.admin_level,
                        character.player.clone(),
                        character.hidden,
                        character.ghost_ticks,
                    ),
                    None => return,
                };

            let previous_coords = coords;
            let coords = get_next_coords(&coords, direction, self.file.width, self.file.height);
            let is_tile_walkable = i32::from(admin_level) >= 1
                || (self.is_tile_walkable(&coords)
                    && (!self.is_tile_occupied(&coords) || ghost_ticks == 0));

            if !self.is_in_bounds(coords) || !is_tile_walkable {
                return;
            }

            Some((previous_coords, coords, player, hidden))
        } {
            if let Some(character) = self.characters.get_mut(&player_id) {
                character.coords = coords;
                character.direction = direction;
                character.entered_coord();
                character.warp_suck_ticks = SETTINGS.world.warp_suck_rate;
                character.ghost_ticks = SETTINGS.world.ghost_rate;
            }

            if let Some(warp) = self.get_warp(&coords) {
                let character = match self.characters.get(&player_id) {
                    Some(character) => character,
                    None => return,
                };

                let player = match character.player.as_ref() {
                    Some(player) => player,
                    None => return,
                };

                if warp.level_required > character.level {
                    return;
                }

                if warp.door > 0 {
                    let door = match self.doors.iter().find(|door| door.coords == coords) {
                        Some(door) => door,
                        None => return,
                    };

                    if !door.open {
                        return;
                    }
                }

                player.request_warp(
                    warp.destination_map,
                    warp.destination_coords,
                    character.map_id == warp.destination_map,
                    None,
                );

                return;
            }

            let packet = {
                let mut packet = WalkReplyServerPacket::default();

                for (other_id, character) in self.characters.iter() {
                    if *other_id != player_id
                        && !character.hidden
                        && in_client_range(&coords, &character.coords)
                        && !in_client_range(&previous_coords, &character.coords)
                    {
                        packet.player_ids.push(*other_id);
                    }
                }
                for (index, item) in self.items.iter() {
                    if in_client_range(&coords, &item.coords)
                        && !in_client_range(&previous_coords, &item.coords)
                    {
                        packet.items.push(item.to_map_info(index));
                    }
                }
                for (index, npc) in self.npcs.iter() {
                    if npc.alive
                        && in_client_range(&coords, &npc.coords)
                        && !in_client_range(&previous_coords, &npc.coords)
                    {
                        packet.npc_indexes.push(*index);
                    }
                }
                packet
            };

            player
                .as_ref()
                .unwrap()
                .send(PacketAction::Reply, PacketFamily::Walk, &packet);

            if !hidden {
                let walk_packet = WalkPlayerServerPacket {
                    player_id,
                    direction,
                    coords,
                };

                self.send_packet_near_player(
                    player_id,
                    PacketAction::Player,
                    PacketFamily::Walk,
                    &walk_packet,
                );

                if let Some(tile) = self.get_tile(&coords) {
                    if matches!(tile, MapTileSpec::Spikes | MapTileSpec::HiddenSpikes) {
                        self.spike_damage(player_id)
                    }
                }
            }

            if coords != client_coords {
                self.request_refresh(player_id);
            }
        }
    }
}
