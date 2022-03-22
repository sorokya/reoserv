use std::collections::HashMap;

use eo::{
    data::{map::MapFile, EOChar, EOShort, EOThree, Serializeable},
    net::{
        packets::server::{avatar, door, face, map_info, players, walk},
        Action, CharacterMapInfo, Family, NearbyInfo, NpcMapInfo,
    },
    world::{Direction, TinyCoords},
};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use crate::character::Character;

use super::{
    get_new_viewable_coords, get_warp_at, is_in_bounds, is_tile_walkable, Command, Item, Npc,
};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: MapFile,
    items: Vec<Item>,
    npcs: Vec<Npc>,
    characters: HashMap<EOShort, Character>,
}

impl Map {
    pub fn new(file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file,
            rx,
            items: Vec::new(),
            npcs: Vec::new(),
            characters: HashMap::new(),
        }
    }

    async fn enter(&mut self, new_character: Box<Character>, respond_to: oneshot::Sender<()>) {
        let character_map_info = new_character.to_map_info();
        let packet = players::Agree::new(character_map_info);
        let buf = packet.serialize();
        for character in self.characters.values() {
            if new_character.is_in_range(character.coords) {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    Action::Agree,
                    Family::Players,
                    buf.clone(),
                );
            }
        }
        self.characters
            .insert(new_character.player_id.unwrap(), *new_character);
        let _ = respond_to.send(());
    }

    async fn face(&mut self, target_player_id: EOShort, direction: Direction) {
        {
            let mut target = self.characters.get_mut(&target_player_id).unwrap();
            target.direction = direction;
        }

        let packet = face::Player::new(target_player_id, direction);
        let buf = packet.serialize();
        let target = self.characters.get(&target_player_id).unwrap();
        for character in self.characters.values() {
            if target_player_id != character.player_id.unwrap()
                && target.is_in_range(character.coords)
            {
                debug!("Send: {:?}", packet);
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .send(Action::Player, Family::Face, buf.clone());
            }
        }
    }

    async fn get_map_info(
        &self,
        player_ids: Option<Vec<EOShort>>,
        _npc_indexes: Option<Vec<EOChar>>,
        respond_to: oneshot::Sender<map_info::Reply>,
    ) {
        let characters = {
            if let Some(player_ids) = player_ids {
                let mut character_infos = Vec::with_capacity(player_ids.len());
                for player_id in player_ids {
                    if let Some(character) = self.characters.get(&player_id) {
                        if !character_infos
                            .iter()
                            .any(|c: &CharacterMapInfo| c.id == player_id)
                        {
                            character_infos.push(character.to_map_info());
                        }
                    }
                }
                Some(character_infos)
            } else {
                None
            }
        };
        let npcs: Option<Vec<NpcMapInfo>> = None; // TODO
        let reply = map_info::Reply { characters, npcs };
        let _ = respond_to.send(reply);
    }

    async fn open_door(&self, target_player_id: EOShort, door_coords: TinyCoords) {
        let target = self.characters.get(&target_player_id).unwrap();
        let coords = door_coords.to_coords();
        if target.is_in_range(coords) {
            let packet = door::Open::new(door_coords.x, door_coords.y);
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.is_in_range(coords) {
                    character.player.as_ref().unwrap().send(
                        Action::Open,
                        Family::Door,
                        buf.clone(),
                    );
                }
            }
        }
    }

    async fn walk(
        &mut self,
        target_player_id: EOShort,
        _timestamp: EOThree,
        _coords: TinyCoords,
        direction: Direction,
    ) {
        if let Some((target_coords, target_player)) = {
            if let Some(target) = self.characters.get_mut(&target_player_id) {
                let mut coords = target.coords;
                match direction {
                    Direction::Up => coords.y -= 1,
                    Direction::Down => coords.y += 1,
                    Direction::Left => coords.x -= 1,
                    Direction::Right => coords.x += 1,
                }
                target.direction = direction;

                let is_tile_walkable = target.admin_level as EOChar >= 1
                    || is_tile_walkable(coords, &self.file.tile_rows);
                if is_in_bounds(
                    coords,
                    self.file.width as EOShort,
                    self.file.height as EOShort,
                ) && is_tile_walkable
                {
                    target.coords = coords;
                }

                Some((target.coords, target.player.clone()))
            } else {
                None
            }
        } {
            // TODO: Ghost timer check
            if let Some(warp) = get_warp_at(target_coords, &self.file.warp_rows) {
                // TODO verify warp requirements
                if let Some(target) = self.characters.get_mut(&target_player_id) {
                    target.player.as_ref().unwrap().request_warp(
                        warp.warp_map,
                        TinyCoords {
                            x: warp.warp_x,
                            y: warp.warp_y,
                        },
                        target.map_id == warp.warp_map,
                    );
                }
            } else {
                let new_viewable_coords = get_new_viewable_coords(
                    target_coords,
                    direction,
                    self.file.width as EOShort,
                    self.file.height as EOShort,
                );

                if !new_viewable_coords.is_empty() {
                    let packet = {
                        let mut packet = walk::Reply::default();
                        for coords in new_viewable_coords {
                            for (player_id, character) in self.characters.iter() {
                                if character.coords == coords {
                                    packet.player_ids.push(*player_id);
                                }
                            }
                            // TODO: items
                            // TODO: npcs
                        }
                        packet
                    };

                    debug!("Send: {:?}", packet);
                    target_player.as_ref().unwrap().send(
                        Action::Reply,
                        Family::Walk,
                        packet.serialize(),
                    );
                }
            }

            let walk_packet = walk::Player {
                player_id: target_player_id,
                direction,
                coords: target_coords.to_tiny_coords(),
            };
            let walk_packet_buf = walk_packet.serialize();
            for (player_id, character) in self.characters.iter() {
                if target_player_id != *player_id && character.is_in_range(target_coords) {
                    debug!("Send: {:?}", walk_packet);
                    character.player.as_ref().unwrap().send(
                        Action::Player,
                        Family::Walk,
                        walk_packet_buf.clone(),
                    );
                }
            }
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Enter(new_character, respond_to) => {
                self.enter(new_character, respond_to).await
            }
            Command::Face(target_player_id, direction) => {
                self.face(target_player_id, direction).await
            }
            Command::GetMapInfo {
                player_ids,
                npc_indexes,
                respond_to,
            } => self.get_map_info(player_ids, npc_indexes, respond_to).await,
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => {
                let target = self.characters.get(&target_player_id).unwrap();
                let mut nearby_items = Vec::new();
                let mut nearby_npcs = Vec::new();
                let mut nearby_characters = Vec::new();
                for item in self.items.iter() {
                    if target.is_in_range(item.coords) {
                        nearby_items.push(item.to_item_map_info());
                    }
                }
                for npc in self.npcs.iter() {
                    if target.is_in_range(npc.coords.to_coords()) {
                        nearby_npcs.push(npc.to_npc_map_info());
                    }
                }
                for character in self.characters.values() {
                    if target_player_id == character.player_id.unwrap()
                        || target.is_in_range(character.coords)
                    {
                        nearby_characters.push(character.to_map_info());
                    }
                }
                let _ = respond_to.send(NearbyInfo {
                    items: nearby_items,
                    npcs: nearby_npcs,
                    characters: nearby_characters,
                });
            }
            Command::GetRidAndSize { respond_to } => {
                let _ = respond_to.send((self.file.rid, self.file.size));
            }
            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => {
                let target = self.characters.remove(&target_player_id).unwrap();
                let packet = avatar::Remove {
                    player_id: target_player_id,
                    warp_animation,
                };
                let buf = packet.serialize();
                for character in self.characters.values() {
                    if target.is_in_range(character.coords) {
                        debug!("Send: {:?}", packet);
                        character.player.as_ref().unwrap().send(
                            Action::Remove,
                            Family::Avatar,
                            buf.clone(),
                        );
                    }
                }
                let _ = respond_to.send(target);
            }
            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords).await,
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
            Command::Walk {
                target_player_id,
                timestamp,
                coords,
                direction,
            } => {
                self.walk(target_player_id, timestamp, coords, direction)
                    .await
            }
        }
    }
}
