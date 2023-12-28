use eolib::{protocol::{Direction, Coords, net::{server::{WalkReplyServerPacket, WalkPlayerServerPacket}, PacketAction, PacketFamily}, map::MapTileSpec}, data::{EoWriter, EoSerialize}};

use crate::utils::{get_next_coords, in_client_range};

use super::super::Map;

// TODO: this function is sooooooooo ugly. Please refactor it
// TODO: force refresh if client out of sync
// TODO: enforce timestamp
impl Map {
    pub fn walk(
        &mut self,
        target_player_id: i32,
        direction: Direction,
        _coords: Coords,
        _timestamp: i32,
    ) {
        if let Some((target_previous_coords, target_coords, target_player, target_hidden)) = {
            let (coords, admin_level, player, hidden) = match self.characters.get(&target_player_id)
            {
                Some(character) => (
                    character.coords,
                    character.admin_level,
                    character.player.clone(),
                    character.hidden,
                ),
                None => return,
            };

            let previous_coords = coords;
            let coords = get_next_coords(&coords, direction, self.file.width, self.file.height);
            let is_tile_walkable = i32::from(admin_level) >= 1
                || (self.is_tile_walkable(&coords) && !self.is_tile_occupied(&coords));
            if !self.is_in_bounds(coords) || !is_tile_walkable {
                return;
            }

            Some((previous_coords, coords, player, hidden))
        } {
            if let Some(character) = self.characters.get_mut(&target_player_id) {
                character.coords = target_coords;
                character.direction = direction;
            }

            // TODO: Ghost timer check
            if let Some(warp) = self.get_warp(&target_coords) {
                let target = match self.characters.get(&target_player_id) {
                    Some(character) => character,
                    None => return,
                };

                if warp.level_required > target.level {
                    return;
                }

                if warp.door > 0 {
                    let door = match self.doors.iter().find(|door| door.coords == target_coords) {
                        Some(door) => door,
                        None => return,
                    };

                    if !door.open {
                        return;
                    }
                }

                target.player.as_ref().unwrap().request_warp(
                    warp.destination_map,
                    warp.destination_coords,
                    target.map_id == warp.destination_map,
                    None,
                );

                return;
            }

            let packet = {
                let mut packet = WalkReplyServerPacket::default();

                for (player_id, character) in self.characters.iter() {
                    if *player_id != target_player_id
                        && !character.hidden
                        && in_client_range(&target_coords, &character.coords)
                        && !in_client_range(&target_previous_coords, &character.coords)
                    {
                        packet.player_ids.push(*player_id);
                    }
                }
                for (index, item) in self.items.iter() {
                    if in_client_range(&target_coords, &item.coords)
                        && !in_client_range(&target_previous_coords, &item.coords)
                    {
                        packet.items.push(item.to_item_map_info(*index));
                    }
                }
                for (index, npc) in self.npcs.iter() {
                    if in_client_range(&target_coords, &npc.coords)
                        && !in_client_range(&target_previous_coords, &npc.coords)
                    {
                        packet.npc_indexes.push(*index);
                    }
                }
                packet
            };

            let mut writer = EoWriter::new();
            packet.serialize(&mut writer);
            target_player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::Walk,
                writer.to_byte_array(),
            );

            if !target_hidden {
                let walk_packet = WalkPlayerServerPacket {
                    player_id: target_player_id,
                    direction,
                    coords: target_coords,
                };

                self.send_packet_near_player(
                    target_player_id,
                    PacketAction::Player,
                    PacketFamily::Walk,
                    walk_packet,
                );

                if let Some(tile) = self.get_tile(&target_coords) {
                    if matches!(tile, MapTileSpec::Spikes | MapTileSpec::HiddenSpikes) {
                        self.spike_damage(target_player_id)
                    }
                }
            }
        }
    }
}
