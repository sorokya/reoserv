use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{server::walk, Direction, PacketAction, PacketFamily},
};

use crate::{
    map::{get_warp_at, is_in_bounds},
    utils::{in_client_range, in_range},
};

use super::Map;

// TODO: this function is sooooooooo ugly. Please refactor it
impl Map {
    pub fn walk(&mut self, target_player_id: EOShort, direction: Direction) {
        if let Some((target_previous_coords, target_coords, target_player)) = {
            let (coords, admin_level, player) = match self.characters.get(&target_player_id) {
                Some(character) => (
                    character.coords,
                    character.admin_level,
                    character.player.clone(),
                ),
                None => return,
            };

            let previous_coords = coords;
            let mut coords = coords;
            match direction {
                Direction::Up => coords.y -= 1,
                Direction::Down => coords.y += 1,
                Direction::Left => coords.x -= 1,
                Direction::Right => coords.x += 1,
            }

            let is_tile_walkable = admin_level as EOChar >= 1 || self.is_tile_walkable(&coords);
            if !is_in_bounds(coords, self.file.width, self.file.height) || !is_tile_walkable {
                return;
            }

            Some((previous_coords, coords, player))
        } {
            if let Some(character) = self.characters.get_mut(&target_player_id) {
                character.coords = target_coords;
                character.direction = direction;
            }

            // TODO: Ghost timer check
            if let Some(warp) = get_warp_at(target_coords, &self.file.warp_rows) {
                // TODO: verify warp requirements
                if let Some(target) = self.characters.get_mut(&target_player_id) {
                    target.player.as_ref().unwrap().request_warp(
                        warp.map,
                        warp.coords,
                        target.map_id == warp.map,
                        None,
                    );
                }
            } else {
                let packet = {
                    let mut packet = walk::Reply::default();

                    for (player_id, character) in self.characters.iter() {
                        if *player_id != target_player_id
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

                debug!("Send: {:?}", packet);
                let mut builder = StreamBuilder::new();
                packet.serialize(&mut builder);
                target_player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Walk,
                    builder.get(),
                );
            }

            let walk_packet = walk::Player {
                player_id: target_player_id,
                direction,
                coords: target_coords,
            };

            debug!("Send: {:?}", walk_packet);
            let mut builder = StreamBuilder::new();
            walk_packet.serialize(&mut builder);
            let walk_packet_buf = builder.get();
            for (player_id, character) in self.characters.iter() {
                if target_player_id != *player_id && in_range(&character.coords, &target_coords) {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Walk,
                        walk_packet_buf.clone(),
                    );
                }
            }
        }
    }
}
