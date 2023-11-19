use std::collections::HashMap;

use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::Coords,
    pubs::{EmfFile, EmfTileSpec},
};
use mysql_async::Pool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{character::Character, world::WorldHandle};

use super::{create_chests, Chest, Command, Door, Item, Npc};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    world: WorldHandle,
    id: EOShort,
    file: EmfFile,
    file_size: EOInt,
    chests: Vec<Chest>,
    doors: Vec<Door>,
    items: HashMap<EOShort, Item>,
    npcs: HashMap<EOChar, Npc>,
    npcs_initialized: bool,
    characters: HashMap<EOShort, Character>,
    pool: Pool,
    quake_ticks: EOInt,
    quake_rate: Option<EOInt>,
    quake_strength: Option<EOInt>,
    has_timed_spikes: bool,
}

mod accept_trade;
mod accept_trade_request;
mod act_npcs;
mod add_chest_item;
mod add_locker_item;
mod add_trade_item;
mod attack;
mod attack_npc_replies;
mod buy_item;
mod cancel_trade;
mod cast_spell;
mod complete_trade;
mod craft_item;
mod create_board_post;
mod deposit_gold;
mod drop_item;
mod emote;
mod enter;
mod equip;
mod face;
mod forget_skill;
mod get_adjacent_tiles;
mod get_character;
mod get_dimensions;
mod get_item;
mod get_map_info;
mod get_nearby_info;
mod get_next_item_index;
mod get_rid_and_size;
mod get_tile;
mod get_warp;
mod give_experience;
mod give_item;
mod is_tile_occupied;
mod is_tile_walkable;
mod is_tile_walkable_npc;
mod junk_item;
mod learn_skill;
mod leave;
mod level_stat;
mod open_bank;
mod open_board;
mod open_chest;
mod open_door;
mod open_inn;
mod open_locker;
mod open_shop;
mod open_skill_master;
mod party_request;
mod play_effect;
mod player_in_range_of_tile;
mod recover_npcs;
mod recover_players;
mod remove_board_post;
mod remove_citizenship;
mod remove_trade_item;
mod request_citizenship;
mod request_paperdoll;
mod request_sleep;
mod request_trade;
mod reset_character;
mod save;
mod sell_item;
mod send_chat_message;
mod send_packet_near;
mod send_packet_near_exclude_player;
mod send_packet_near_player;
mod send_trade_update;
mod serialize;
mod sit;
mod sit_chair;
mod sleep;
mod spawn_items;
mod spawn_npcs;
mod spike_damage;
mod stand;
mod start_spell_chant;
mod take_chest_item;
mod take_locker_item;
mod timed_door_close;
mod timed_drain;
mod timed_quake;
mod timed_spikes;
mod timed_warp_suck;
mod toggle_hidden;
mod unaccept_trade;
mod unequip;
mod upgrade_locker;
mod use_item;
mod view_board_post;
mod walk;
mod withdraw_gold;

impl Map {
    pub fn new(
        id: EOShort,
        file_size: EOInt,
        file: EmfFile,
        pool: Pool,
        world: WorldHandle,
        rx: UnboundedReceiver<Command>,
    ) -> Self {
        let has_timed_spikes = file.spec_rows.iter().any(|row| {
            row.tiles
                .iter()
                .any(|tile| tile.spec == EmfTileSpec::TimedSpikes)
        });

        let mut doors: Vec<Door> = Vec::new();
        for row in &file.warp_rows {
            for tile in &row.tiles {
                if tile.warp.door > 0 {
                    doors.push(Door::new(
                        Coords {
                            x: tile.x,
                            y: row.y,
                        },
                        tile.warp.door,
                    ));
                }
            }
        }

        let chests = create_chests(id, &file);

        Self {
            id,
            world,
            file_size,
            file,
            rx,
            chests,
            doors,
            items: HashMap::new(),
            npcs: HashMap::new(),
            npcs_initialized: false,
            characters: HashMap::new(),
            pool,
            quake_ticks: 0,
            quake_rate: None,
            quake_strength: None,
            has_timed_spikes,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::AcceptTrade { player_id } => self.accept_trade(player_id).await,
            Command::AcceptTradeRequest {
                player_id,
                target_player_id,
            } => self.accept_trade_request(player_id, target_player_id).await,
            Command::AddChestItem { player_id, item } => self.add_chest_item(player_id, item).await,
            Command::AddLockerItem { player_id, item } => {
                self.add_locker_item(player_id, item).await
            }
            Command::AddTradeItem { player_id, item } => self.add_trade_item(player_id, item).await,
            Command::Attack {
                target_player_id,
                direction,
                timestamp,
            } => self.attack(target_player_id, direction, timestamp).await,

            Command::BuyItem {
                player_id,
                item,
                session_id,
            } => self.buy_item(player_id, item, session_id).await,

            Command::CancelTrade {
                player_id,
                partner_player_id,
            } => self.cancel_trade(player_id, partner_player_id),

            Command::CastSpell { player_id, target } => self.cast_spell(player_id, target).await,

            Command::CraftItem {
                player_id,
                item_id,
                session_id,
            } => self.craft_item(player_id, item_id, session_id).await,

            Command::CreateBoardPost {
                player_id,
                subject,
                body,
            } => self.create_board_post(player_id, subject, body).await,

            Command::DepositGold {
                player_id,
                session_id,
                amount,
            } => self.deposit_gold(player_id, session_id, amount).await,

            Command::DropItem {
                target_player_id,
                item,
                coords,
            } => self.drop_item(target_player_id, item, coords).await,

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
            } => self.equip(player_id, item_id, sub_loc).await,

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

            Command::GetRelogCoords { respond_to } => {
                let _ = respond_to.send(if self.file.relog_x > 0 {
                    Some(Coords {
                        x: self.file.relog_x,
                        y: self.file.relog_y,
                    })
                } else {
                    None
                });
            }

            Command::GetRidAndSize { respond_to } => {
                self.get_rid_and_size(respond_to);
            }

            Command::GiveItem {
                target_player_id,
                item_id,
                amount,
            } => self.give_item(target_player_id, item_id, amount),

            Command::HasPlayer {
                player_id,
                respond_to,
            } => {
                let _ = respond_to.send(self.characters.contains_key(&player_id));
            }

            Command::JunkItem {
                target_player_id,
                item_id,
                amount,
            } => self.junk_item(target_player_id, item_id, amount).await,

            Command::LearnSkill {
                player_id,
                spell_id,
                session_id,
            } => self.learn_skill(player_id, spell_id, session_id).await,

            Command::Leave {
                player_id,
                warp_animation,
                respond_to,
                interact_player_id,
            } => {
                self.leave(player_id, warp_animation, respond_to, interact_player_id);
            }

            Command::LevelStat { player_id, stat_id } => self.level_stat(player_id, stat_id),

            Command::OpenBank {
                player_id,
                npc_index,
            } => self.open_bank(player_id, npc_index).await,

            Command::OpenBoard {
                player_id,
                board_id,
            } => self.open_board(player_id, board_id),

            Command::OpenChest { player_id, coords } => self.open_chest(player_id, coords),

            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords),

            Command::OpenInn {
                player_id,
                npc_index,
            } => self.open_inn(player_id, npc_index).await,

            Command::OpenLocker { player_id } => self.open_locker(player_id),

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

            Command::RemoveBoardPost { player_id, post_id } => {
                self.remove_board_post(player_id, post_id).await
            }

            Command::RemoveCitizenship { player_id } => self.remove_citizenship(player_id).await,

            Command::RemoveTradeItem { player_id, item_id } => {
                self.remove_trade_item(player_id, item_id).await
            }

            Command::RequestCitizenship {
                player_id,
                session_id,
                answers,
            } => {
                self.request_citizenship(player_id, session_id, answers)
                    .await
            }

            Command::RequestPaperdoll {
                player_id,
                target_player_id,
            } => self.request_paperdoll(player_id, target_player_id).await,

            Command::RequestSleep {
                player_id,
                session_id,
            } => self.request_sleep(player_id, session_id).await,

            Command::PartyRequest {
                target_player_id,
                request,
            } => self.party_request(target_player_id, request).await,

            Command::RequestTrade {
                player_id,
                target_player_id,
            } => self.request_trade(player_id, target_player_id),

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

            Command::Sleep {
                player_id,
                session_id,
            } => self.sleep(player_id, session_id).await,

            Command::Stand { player_id } => self.stand(player_id),

            Command::StartSpellChant {
                player_id,
                spell_id,
                timestamp,
            } => self.start_spell_chant(player_id, spell_id, timestamp),

            Command::SpawnItems => self.spawn_items().await,

            Command::SpawnNpcs => self.spawn_npcs().await,

            Command::TakeChestItem { player_id, item_id } => {
                self.take_chest_item(player_id, item_id).await;
            }

            Command::TakeLockerItem { player_id, item_id } => {
                self.take_locker_item(player_id, item_id)
            }

            Command::TimedDoorClose => self.timed_door_close(),

            Command::TimedDrain => self.timed_drain(),

            Command::TimedQuake => self.timed_quake(),

            Command::TimedSpikes => self.timed_spikes(),

            Command::TimedWarpSuck => self.timed_warp_suck(),

            Command::ToggleHidden { player_id } => self.toggle_hidden(player_id),

            Command::ActNpcs => self.act_npcs(),

            Command::UnacceptTrade { player_id } => self.unaccept_trade(player_id).await,

            Command::Unequip {
                player_id,
                item_id,
                sub_loc,
            } => self.unequip(player_id, item_id, sub_loc),

            Command::UpgradeLocker { player_id } => self.upgrade_locker(player_id),

            Command::UseItem { player_id, item_id } => self.use_item(player_id, item_id).await,

            Command::ViewBoardPost { player_id, post_id } => {
                self.view_board_post(player_id, post_id).await
            }

            Command::Walk {
                target_player_id,
                direction,
                coords,
                timestamp,
            } => self.walk(target_player_id, direction, coords, timestamp),

            Command::WithdrawGold {
                player_id,
                session_id,
                amount,
            } => self.withdraw_gold(player_id, session_id, amount).await,
        }
    }
}
