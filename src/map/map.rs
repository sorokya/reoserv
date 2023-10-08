use std::collections::HashMap;

use eo::{
    data::{EOChar, EOInt, EOShort},
    pubs::EmfFile,
};
use mysql_async::Pool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::character::Character;

use super::{Chest, Command, Item, Npc};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    _id: EOShort,
    file: EmfFile,
    file_size: EOInt,
    chests: Vec<Chest>,
    items: HashMap<EOShort, Item>,
    npcs: HashMap<EOChar, Npc>,
    characters: HashMap<EOShort, Character>,
    pool: Pool,
}

mod act_npcs;
mod attack;
mod buy_item;
mod craft_item;
mod drop_item;
mod emote;
mod enter;
mod equip;
mod face;
mod forget_skill;
mod get_character;
mod get_dimensions;
mod get_item;
mod get_map_info;
mod get_nearby_info;
mod get_next_item_index;
mod get_rid_and_size;
mod get_tile;
mod give_item;
mod is_tile_occupied;
mod is_tile_walkable;
mod is_tile_walkable_npc;
mod junk_item;
mod learn_skill;
mod leave;
mod level_stat;
mod open_chest;
mod open_door;
mod open_shop;
mod open_skill_master;
mod play_effect;
mod recover_npcs;
mod recover_players;
mod request_paperdoll;
mod reset_character;
mod save;
mod sell_item;
mod send_chat_message;
mod send_packet_near;
mod send_packet_near_exclude_player;
mod send_packet_near_player;
mod serialize;
mod sit;
mod sit_chair;
mod spawn_items;
mod spawn_npcs;
mod stand;
mod take_chest_item;
mod unequip;
mod use_item;
mod walk;

impl Map {
    pub fn new(
        id: EOShort,
        file_size: EOInt,
        file: EmfFile,
        pool: Pool,
        rx: UnboundedReceiver<Command>,
    ) -> Self {
        Self {
            _id: id,
            file_size,
            file,
            rx,
            chests: Vec::new(),
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
                timestamp,
            } => self.attack(target_player_id, direction, timestamp),

            Command::BuyItem {
                player_id,
                item,
                session_id,
            } => self.buy_item(player_id, item, session_id).await,

            Command::CraftItem {
                player_id,
                item_id,
                session_id,
            } => self.craft_item(player_id, item_id, session_id).await,

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

            Command::ForgetSkill {
                player_id,
                skill_id,
                session_id,
            } => self.forget_skill(player_id, skill_id, session_id).await,

            Command::GetCharacter {
                player_id,
                respond_to,
            } => self.get_character(player_id, respond_to),

            Command::GetDimensions { respond_to } => {
                self.get_dimensions(respond_to);
            }

            Command::GetItem {
                target_player_id,
                item_index,
            } => {
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

            Command::LearnSkill {
                player_id,
                spell_id,
                session_id,
            } => self.learn_skill(player_id, spell_id, session_id).await,

            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => self.leave(target_player_id, warp_animation, respond_to),

            Command::LevelStat { player_id, stat_id } => self.level_stat(player_id, stat_id),

            Command::OpenChest { player_id, coords } => self.open_chest(player_id, coords),

            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords),

            Command::OpenShop {
                player_id,
                npc_index,
            } => self.open_shop(player_id, npc_index).await,

            Command::OpenSkillMaster {
                player_id,
                npc_index,
            } => self.open_skill_master(player_id, npc_index).await,

            Command::RecoverNpcs => self.recover_npcs().await,

            Command::RecoverPlayers => self.recover_players().await,

            Command::RequestPaperdoll {
                player_id,
                target_player_id,
            } => self.request_paperdoll(player_id, target_player_id),

            Command::ResetCharacter {
                player_id,
                session_id,
            } => self.reset_character(player_id, session_id).await,

            Command::Save { respond_to } => self.save(respond_to).await,

            Command::SellItem {
                player_id,
                item,
                session_id,
            } => self.sell_item(player_id, item, session_id).await,

            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message),

            Command::Serialize { respond_to } => {
                self.serialize(respond_to);
            }

            Command::Sit { player_id } => self.sit(player_id),

            Command::SitChair { player_id, coords } => self.sit_chair(player_id, coords),

            Command::Stand { player_id } => self.stand(player_id),

            Command::SpawnItems => self.spawn_items().await,

            Command::SpawnNpcs => self.spawn_npcs().await,

            Command::TakeChestItem {
                player_id,
                coords,
                item_id,
            } => {
                self.take_chest_item(player_id, coords, item_id);
            }

            Command::ActNpcs => self.act_npcs(),

            Command::Unequip {
                player_id,
                item_id,
                sub_loc,
            } => self.unequip(player_id, item_id, sub_loc),

            Command::UseItem { player_id, item_id } => self.use_item(player_id, item_id),

            Command::Walk {
                target_player_id,
                direction,
                coords,
                timestamp,
            } => self.walk(target_player_id, direction, coords, timestamp),
        }
    }
}
