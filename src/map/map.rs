use std::collections::HashMap;

use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable},
    pubs::EmfFile,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::character::Character;

use super::{Command, Item, Npc, NpcData};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: EmfFile,
    file_size: EOInt,
    items: Vec<Item>,
    npcs: HashMap<EOChar, Npc>,
    npc_data: HashMap<EOShort, NpcData>,
    characters: HashMap<EOShort, Character>,
}

mod act_npcs;
mod emote;
mod enter;
mod face;
mod get_map_info;
mod get_nearby_info;
mod leave;
mod open_door;
mod send_chat_message;
mod spawn_npcs;
mod walk;

impl Map {
    pub fn new(file_size: EOInt, file: EmfFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file_size,
            file,
            rx,
            items: Vec::new(),
            npcs: HashMap::new(),
            npc_data: HashMap::new(),
            characters: HashMap::new(),
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Emote {
                target_player_id,
                emote,
            } => self.emote(target_player_id, emote),
            Command::Enter {
                character,
                warp_animation,
                respond_to,
            } => self.enter(character, warp_animation, respond_to),
            Command::Face {
                target_player_id,
                direction,
            } => self.face(target_player_id, direction),
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
            } => self.get_map_info(player_ids, npc_indexes, respond_to),
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => self.get_nearby_info(target_player_id, respond_to),
            Command::GetRidAndSize { respond_to } => {
                let _ = respond_to.send((self.file.rid, self.file_size));
            }
            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => self.leave(target_player_id, warp_animation, respond_to),
            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords),
            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message),
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
            Command::SpawnNpcs => self.spawn_npcs().await,
            Command::ActNpcs => self.act_npcs(),
            Command::Walk {
                target_player_id,
                timestamp,
                coords,
                direction,
            } => self.walk(target_player_id, timestamp, coords, direction),
        }
    }
}
