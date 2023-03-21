use std::collections::HashMap;

use eo::{
    data::{EOChar, EOInt, EOShort},
    pubs::EmfFile,
};
use mysql_async::Pool;
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
    pool: Pool,
}

mod act_npcs;
mod emote;
mod enter;
mod face;
mod get_character;
mod get_dimensions;
mod get_map_info;
mod get_nearby_info;
mod get_rid_and_size;
mod leave;
mod open_door;
mod save;
mod send_chat_message;
mod serialize;
mod spawn_npcs;
mod walk;

impl Map {
    pub fn new(file_size: EOInt, file: EmfFile, pool: Pool, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file_size,
            file,
            rx,
            items: Vec::new(),
            npcs: HashMap::new(),
            npc_data: HashMap::new(),
            characters: HashMap::new(),
            pool,
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
            } => self.get_character(player_id, respond_to),

            Command::GetDimensions { respond_to } => {
                self.get_dimensions(respond_to);
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
                self.get_rid_and_size(respond_to);
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

            Command::Save { respond_to } => self.save(respond_to).await,

            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message),

            Command::Serialize { respond_to } => {
                self.serialize(respond_to);
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
