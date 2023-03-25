use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{server::walk, Direction, PacketAction, PacketFamily},
};

use crate::map::{get_warp_at, is_in_bounds, is_tile_walkable};

use super::Map;

impl Map {
    pub fn walk(
        &mut self,
        target_player_id: EOShort,
        direction: Direction,
    ) {
        if let Some((target_previous_coords, target_coords, target_player)) = {
            if let Some(target) = self.characters.get_mut(&target_player_id) {
                let previous_coords = target.coords;
                let mut coords = target.coords;
                match direction {
                    Direction::Up => coords.y -= 1,
                    Direction::Down => coords.y += 1,
                    Direction::Left => coords.x -= 1,
                    Direction::Right => coords.x += 1,
                }
                target.direction = direction;

                let is_tile_walkable = target.admin_level as EOChar >= 1
                    || is_tile_walkable(coords, &self.file.spec_rows);
                if is_in_bounds(coords, self.file.width, self.file.height) && is_tile_walkable {
                    target.coords = coords;
                }

                Some((previous_coords, target.coords, target.player.clone()))
            } else {
                None
            }
        } {
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
                            && character.is_in_range(&target_coords)
                            && !character.is_in_range(&target_previous_coords)
                        {
                            packet.player_ids.push(*player_id);
                        }
                    }
                    for (index, item) in self.items.iter() {
                        if item.is_in_range(&target_coords)
                            && !item.is_in_range(&target_previous_coords)
                        {
                            packet.items.push(item.to_item_map_info(*index));
                        }
                    }
                    for (index, npc) in self.npcs.iter() {
                        if npc.is_in_range(&target_coords)
                            && !npc.is_in_range(&target_previous_coords)
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
                if target_player_id != *player_id && character.is_in_range(&target_coords) {
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
