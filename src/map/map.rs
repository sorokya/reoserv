use std::{collections::HashMap, sync::Arc};

use eo::{
    data::{map::MapFile, EOChar, EOShort, EOThree, Serializeable},
    net::{
        packets::server::{avatar, face, map_info, players, walk},
        Action, CharacterMapInfo, Family, NearbyInfo, NpcMapInfo,
    },
    world::{Coords, Direction, TinyCoords},
};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot, Mutex};

use crate::{character::Character, SETTINGS};

use super::{get_new_viewable_coords, Command, Item, NPC};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: MapFile,
    items: Arc<Mutex<Vec<Item>>>,
    npcs: Arc<Mutex<Vec<NPC>>>,
    characters: Arc<Mutex<HashMap<EOShort, Character>>>,
}

impl Map {
    pub fn new(file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file,
            rx,
            items: Arc::new(Mutex::new(Vec::new())),
            npcs: Arc::new(Mutex::new(Vec::new())),
            characters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn enter(&mut self, new_character: Character, respond_to: oneshot::Sender<()>) {
        let character_map_info = new_character.to_map_info();
        let packet = players::Agree::new(character_map_info);
        let buf = packet.serialize();
        let mut characters = self.characters.lock().await;
        for character in characters.values() {
            if new_character.is_in_range(character.coords) {
                character.player.as_ref().unwrap().send(
                    Action::Agree,
                    Family::Players,
                    buf.clone(),
                );
            }
        }
        characters.insert(new_character.player_id.unwrap(), new_character);
        let _ = respond_to.send(());
    }

    async fn face(&mut self, target_player_id: EOShort, direction: Direction) {
        {
            let mut characters = self.characters.lock().await;
            let mut target = characters.get_mut(&target_player_id).unwrap();
            target.direction = direction;
        }

        let packet = face::Player::new(target_player_id, direction);
        let buf = packet.serialize();
        let characters = self.characters.lock().await;
        let target = characters.get(&target_player_id).unwrap();
        for character in characters.values() {
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
                let characters = self.characters.lock().await;
                for player_id in player_ids {
                    if let Some(character) = characters.get(&player_id) {
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

    async fn walk(
        &self,
        target_player_id: EOShort,
        timestamp: EOThree,
        _coords: TinyCoords,
        direction: Direction,
    ) {
        if let Some((target_coords, target_player)) = {
            let mut characters = self.characters.lock().await;
            if let Some(target) = characters.get_mut(&target_player_id) {
                match direction {
                    Direction::Up => target.coords.y -= 1,
                    Direction::Down => target.coords.y += 1,
                    Direction::Left => target.coords.x -= 1,
                    Direction::Right => target.coords.x += 1,
                }
                target.direction = direction;
                Some((target.coords, target.player.clone()))
            } else {
                None
            }
        } {
            // TODO: bounds check

            // TODO: Ghost timer check

            // TODO: Warp

            let new_viewable_coords = get_new_viewable_coords(
                target_coords,
                direction,
                self.file.width as EOShort,
                self.file.height as EOShort,
            );

            if new_viewable_coords.len() > 0 {
                let packet = {
                    let mut packet = walk::Reply::default();
                    let characters = self.characters.lock().await;
                    debug!("New coords: {:?}", new_viewable_coords);
                    for coords in new_viewable_coords {
                        for (player_id, character) in characters.iter() {
                            if character.coords == coords {
                                packet.player_ids.push(*player_id);
                            }
                        }
                        // TODO items
                        // TODO npcs
                    }
                    packet
                };

                target_player.as_ref().unwrap().send(
                    Action::Reply,
                    Family::Walk,
                    packet.serialize(),
                );
            }

            let walk_packet = walk::Player {
                player_id: target_player_id,
                direction,
                coords: target_coords.to_tiny_coords(),
            };
            let walk_packet_buf = walk_packet.serialize();
            let characters = self.characters.lock().await;
            for (player_id, character) in characters.iter() {
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
            Command::GetHashAndSize { respond_to } => {
                let _ = respond_to.send((self.file.hash, self.file.size));
            }
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => {
                let characters = self.characters.lock().await;
                let target = characters.get(&target_player_id).unwrap();
                let items = self.items.lock().await;
                let npcs = self.npcs.lock().await;
                let mut nearby_items = Vec::new();
                let mut nearby_npcs = Vec::new();
                let mut nearby_characters = Vec::new();
                for item in items.iter() {
                    if target.is_in_range(item.coords) {
                        nearby_items.push(item.to_item_map_info());
                    }
                }
                for npc in npcs.iter() {
                    if target.is_in_range(npc.coords.to_coords()) {
                        nearby_npcs.push(npc.to_npc_map_info());
                    }
                }
                for character in characters.values() {
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
            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => {
                let mut characters = self.characters.lock().await;
                let target = characters.remove(&target_player_id).unwrap();
                let packet = avatar::Remove {
                    player_id: target_player_id,
                    warp_animation,
                };
                let buf = packet.serialize();
                for character in characters.values() {
                    if target.is_in_range(character.coords) {
                        character.player.as_ref().unwrap().send(
                            Action::Remove,
                            Family::Avatar,
                            buf.clone(),
                        );
                    }
                }
                let _ = respond_to.send(());
            }
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