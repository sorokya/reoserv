use std::{collections::{HashMap, HashSet}, cmp};

use chrono::Duration;
use eo::{
    character::Emote,
    data::{map::{MapFile, NPCSpawn, NPCSpeed}, EOChar, EOShort, EOThree, Serializeable},
    net::{
        packets::server::{avatar, door, emote, face, map_info, players, talk, walk},
        Action, CharacterMapInfo, Family, NearbyInfo, NpcMapInfo,
    },
    world::{Direction, TinyCoords, WarpAnimation},
};
use num_traits::FromPrimitive;
use rand::Rng;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use crate::{character::Character, SETTINGS};

use super::{
    get_new_viewable_coords, get_warp_at, is_in_bounds, is_tile_walkable, Command, Item, Npc, is_occupied,
};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: MapFile,
    items: Vec<Item>,
    npcs: HashMap<EOChar, Npc>,
    characters: HashMap<EOShort, Character>,
}

impl Map {
    pub fn new(file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file,
            rx,
            items: Vec::new(),
            npcs: HashMap::new(),
            characters: HashMap::new(),
        }
    }

    async fn emote(&self, target_player_id: EOShort, emote: Emote) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = emote::Player::new(target_player_id, emote);
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.player_id.unwrap() != target_player_id
                    && character.is_in_range(target.coords)
                {
                    debug!("Send: {:?}", packet);
                    character.player.as_ref().unwrap().send(
                        Action::Player,
                        Family::Emote,
                        buf.clone(),
                    );
                }
            }
        }
    }

    async fn enter(
        &mut self,
        new_character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    ) {
        let mut character_map_info = new_character.to_map_info();
        character_map_info.warp_animation = warp_animation;
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
        npc_indexes: Option<Vec<EOChar>>,
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
        let npcs: Option<Vec<NpcMapInfo>> = {
            if let Some(npc_indexes) = npc_indexes {
                let mut npc_infos = Vec::with_capacity(npc_indexes.len());
                for npc_index in npc_indexes {
                    if let Some(npc) = self.npcs.get(&npc_index) {
                        npc_infos.push(npc.to_map_info(&npc_index));
                    }
                }
                Some(npc_infos)
            } else {
                None
            }
        };
        let reply = map_info::Reply { characters, npcs };
        let _ = respond_to.send(reply);
    }

    async fn open_door(&self, target_player_id: EOShort, door_coords: TinyCoords) {
        let target = self.characters.get(&target_player_id).unwrap();
        if target.is_in_range(door_coords) {
            let packet = door::Open::new(door_coords.x, door_coords.y);
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.is_in_range(door_coords) {
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
                    self.file.width,
                    self.file.height,
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
                // TODO: verify warp requirements
                if let Some(target) = self.characters.get_mut(&target_player_id) {
                    target.player.as_ref().unwrap().request_warp(
                        warp.warp_map,
                        TinyCoords {
                            x: warp.warp_x,
                            y: warp.warp_y,
                        },
                        target.map_id == warp.warp_map,
                        None,
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
                            for item in self.items.iter() {
                                if item.coords == coords {
                                    packet.items.push(item.to_item_map_info());
                                }
                            }
                            for (index, npc) in self.npcs.iter() {
                                if npc.coords == coords {
                                    packet.npc_indexes.push(*index);
                                }
                            }
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
                coords: target_coords,
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

    async fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = talk::Player {
                player_id: target_player_id,
                message,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if target_player_id != character.player_id.unwrap()
                    && target.is_in_range(character.coords)
                {
                    character.player.as_ref().unwrap().send(
                        Action::Player,
                        Family::Talk,
                        buf.clone(),
                    );
                }
            }
        }
    }

    fn spawn_npcs(&mut self) {
        // TODO: test if this is actually how GameServer.exe works
        let now = chrono::Utc::now();
        let mut rng = rand::thread_rng();

        if !self.file.npc_spawns.is_empty() {
            if self.npcs.is_empty() {
                let mut npc_index: EOChar = 0;

                let dead_since = if SETTINGS.npcs.instant_spawn {
                    now - Duration::days(1)
                } else {
                    now
                };

                for (spawn_index, spawn) in self.file.npc_spawns.iter().enumerate() {
                    // TODO: bounds check
                    for _ in 0..spawn.amount {
                        self.npcs.insert(npc_index, Npc::new(spawn.npc_id, TinyCoords::new(0, 0), Direction::Down, spawn_index, dead_since));
                        npc_index += 1;
                    }
                }
            }

            if SETTINGS.npcs.freeze_on_empty_map && self.characters.is_empty() {
                return;
            }

            // get occupied tiles of all characters and npcs
            let mut occupied_tiles = HashSet::new();
            for character in self.characters.values() {
                occupied_tiles.insert(character.coords);
            }
            for npc in self.npcs.values() {
                occupied_tiles.insert(npc.coords);
            }

            for npc in self.npcs.values_mut() {
                let spawn = &self.file.npc_spawns[npc.spawn_index];
                if !npc.alive && now.timestamp() - npc.dead_since.timestamp() > spawn.respawn_time.into() {
                    npc.alive = true;

                    let mut coords = TinyCoords::new(spawn.x, spawn.y);

                    // TODO: break loop after a certain number of tries
                    while is_occupied(coords, &occupied_tiles) || !is_tile_walkable(coords, &self.file.tile_rows) {
                        // set position to random spot in a radius of 3
                        coords.x = cmp::max(0, rng.gen_range(coords.x as i32 - 3..coords.x as i32 + 3)) as EOChar;
                        coords.y = cmp::max(0, rng.gen_range(coords.y as i32 - 3..coords.y as i32 + 3)) as EOChar;
                    }

                    npc.coords = coords;
                    npc.direction = if spawn.speed == NPCSpeed::Frozen {
                        Direction::from_u16(spawn.respawn_time & 0x03).unwrap()
                    } else {
                        match rand::random::<u8>() % 4 {
                            0 => Direction::Down,
                            1 => Direction::Left,
                            2 => Direction::Up,
                            3 => Direction::Right,
                            _ => unreachable!(),
                        }
                    };
                    occupied_tiles.insert(coords);
                }
            }
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Emote {
                target_player_id,
                emote,
            } => self.emote(target_player_id, emote).await,
            Command::Enter {
                character,
                warp_animation,
                respond_to,
            } => self.enter(character, warp_animation, respond_to).await,
            Command::Face {
                target_player_id,
                direction,
            } => self.face(target_player_id, direction).await,
            Command::GetCharacter {
                player_id,
                respond_to,
            } => {
                if let Some(character) = self.characters.get(&player_id) {
                    let _ = respond_to.send(Some(Box::new(character.to_owned())));
                } else {
                    let _ = respond_to.send(None);
                }
            }
            Command::GetDimensions { respond_to } => {
                let _ = respond_to.send((self.file.width, self.file.height));
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
                for (index, npc) in self.npcs.iter() {
                    if target.is_in_range(npc.coords) {
                        nearby_npcs.push(npc.to_map_info(index));
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
            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message).await,
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
            Command::SpawnNpcs => self.spawn_npcs(),
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
