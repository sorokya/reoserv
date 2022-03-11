use std::{collections::HashMap, sync::Arc};

use eo::{
    data::{map::MapFile, EOShort, Serializeable},
    net::{
        packets::server::{avatar, face, map_info, players},
        Action, Family, NearbyInfo, CharacterMapInfo, NpcMapInfo,
    },
};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::{character::Character};

use super::{Command, Item, NPC};

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

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Enter(new_character, respond_to) => {
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
            Command::Face(target_player_id, direction) => {
                let packet = face::Player::new(target_player_id, direction);
                let buf = packet.serialize();
                let characters = self.characters.lock().await;
                let target = characters.get(&target_player_id).unwrap();
                for character in characters.values() {
                    if target_player_id != character.player_id.unwrap()
                        && target.is_in_range(character.coords)
                    {
                        character.player.as_ref().unwrap().send(
                            Action::Player,
                            Family::Face,
                            buf.clone(),
                        );
                    }
                }
            }
            Command::GetMapInfo {
                player_ids,
                _npc_indexes,
                respond_to,
            } => {
                let characters = {
                    if let Some(player_ids) = player_ids {
                        let mut character_infos = Vec::with_capacity(player_ids.len());
                        let characters = self.characters.lock().await;
                        for player_id in player_ids {
                            if let Some(character) = characters.get(&player_id) {
                                if !character_infos.iter().any(|c: &CharacterMapInfo| c.id == player_id) {
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
                let reply = map_info::Reply {
                    characters,
                    npcs,
                };
                let _ = respond_to.send(Ok(reply));
            }
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
        }
    }
}
