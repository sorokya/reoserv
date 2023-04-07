use std::collections::HashMap;

use eo::{
    data::{EOChar, EOInt, EOShort},
    pubs::EmfFile,
};
use mysql_async::Pool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::character::Character;

use super::{Command, Item, Npc};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: EmfFile,
    file_size: EOInt,
    items: HashMap<EOShort, Item>,
    npcs: HashMap<EOChar, Npc>,
    characters: HashMap<EOShort, Character>,
    pool: Pool,
}

mod act_npcs;
mod attack;
mod drop_item;
mod emote;
mod enter;
mod equip;
mod face;
mod get_character;
mod get_dimensions;
mod get_item;
mod get_map_info;
mod get_nearby_info;
mod get_next_item_index;
mod get_rid_and_size;
mod give_item;
mod junk_item;
mod leave;
mod open_door;
mod play_effect;
mod request_paperdoll;
mod save;
mod send_chat_message;
mod send_packet_near;
mod send_packet_near_player;
mod serialize;
mod spawn_npcs;
mod unequip;
mod use_item;
mod walk;

impl Map {
    pub fn new(file_size: EOInt, file: EmfFile, pool: Pool, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file_size,
            file,
            rx,
            items: HashMap::new(),
            npcs: HashMap::new(),
            characters: HashMap::new(),
            pool,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Attack {
                target_player_id,
                direction,
            } => self.attack(target_player_id, direction),

            Command::DropItem {
                target_player_id,
                item,
                coords,
            } => self.drop_item(target_player_id, item, coords),

            Command::Emote {
                target_player_id,
                emote,
            } => self.emote(target_player_id, emote),

            Command::Enter {
                character,
                warp_animation,
                respond_to,
            } => self.enter(character, warp_animation, respond_to),

            Command::Equip {
                player_id,
                item_id,
                sub_loc,
            } => self.equip(player_id, item_id, sub_loc),

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

            Command::GetItem { target_player_id, item_index } => {
                self.get_item(target_player_id, item_index);
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

            Command::GiveItem {
                target_player_id,
                item_id,
                amount,
            } => self.give_item(target_player_id, item_id, amount),

            Command::JunkItem {
                target_player_id,
                item_id,
                amount,
            } => self.junk_item(target_player_id, item_id, amount),

            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => self.leave(target_player_id, warp_animation, respond_to),

            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords),

            Command::RequestPaperdoll {
                player_id,
                target_player_id,
            } => self.request_paperdoll(player_id, target_player_id),

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

            Command::Unequip {
                player_id,
                item_id,
                sub_loc,
            } => self.unequip(player_id, item_id, sub_loc),

            Command::UseItem {
                player_id,
                item_id,
            } => self.use_item(player_id, item_id),

            Command::Walk {
                target_player_id,
                direction,
            } => self.walk(target_player_id, direction),
        }
    }
}
