use std::collections::HashMap;

use eolib::protocol::{map::{Emf, MapTileSpec}, Coords};
use mysql_async::Pool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{character::Character, world::WorldHandle};

use super::{Chest, Command, Door, Item, Npc};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    world: WorldHandle,
    id: i32,
    file: Emf,
    file_size: i32,
    chests: Vec<Chest>,
    doors: Vec<Door>,
    items: HashMap<i32, Item>,
    npcs: HashMap<i32, Npc>,
    npcs_initialized: bool,
    characters: HashMap<i32, Character>,
    pool: Pool,
    quake_ticks: i32,
    arena_ticks: i32,
    arena_players: Vec<ArenaPlayer>,
    quake_rate: Option<i32>,
    quake_strength: Option<i32>,
    has_timed_spikes: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ArenaPlayer {
    pub player_id: i32,
    pub kills: i32,
}

mod bank;
mod board;
mod character;
mod chest;
mod events;
mod inn;
mod locker;
mod shop;
mod skill_master;
mod trade;
mod utils;

impl Map {
    pub fn new(
        id: i32,
        file_size: i32,
        file: Emf,
        pool: Pool,
        world: WorldHandle,
        rx: UnboundedReceiver<Command>,
    ) -> Self {
        let has_timed_spikes = file.tile_spec_rows.iter().any(|row| {
            row.tiles
                .iter()
                .any(|tile| tile.tile_spec == MapTileSpec::TimedSpikes)
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

        let chests = utils::create_chests(id, &file);

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
            arena_ticks: 0,
            arena_players: Vec::new(),
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
            Command::AgreeTrade { player_id: _ } => todo!(),
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

            Command::DisagreeTrade { player_id: _ } => todo!(),

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

            Command::TimedArena => self.timed_arena(),

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
            Command::FindPlayer { player_id: _, name: _ } => todo!(),
            Command::RequestNpcs { player_id: _, npc_indexes: _ } => todo!(),
            Command::RequestPlayers { player_id: _, player_ids: _ } => todo!(),
            Command::RequestPlayersAndNpcs { player_id: _, player_ids: _, npc_indexes: _ } => todo!(),
            Command::RequestRefresh { player_id: _ } => todo!(),
        }
    }
}
