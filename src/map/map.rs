use std::collections::HashMap;

use eolib::protocol::{
    map::{Emf, MapTileSpec},
    Coords,
};
use mysql_async::Pool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{character::Character, world::WorldHandle, SETTINGS};

use super::{Chest, Command, Door, Item, Npc, Wedding};

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
    jukebox_player: Option<String>,
    jukebox_ticks: i32,
    has_jukebox: bool,
    wedding: Option<Wedding>,
    wedding_ticks: i32,
    evacuate_ticks: Option<i32>,
}

#[derive(Debug, Copy, Clone)]
pub struct ArenaPlayer {
    pub player_id: i32,
    pub kills: i32,
}

mod bank;
mod barber;
mod board;
mod character;
mod chest;
mod events;
#[macro_use]
mod guild;
mod effect;
mod inn;
mod jukebox;
mod locker;
mod marriage;
mod quest;
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

        let has_jukebox = file.tile_spec_rows.iter().any(|row| {
            row.tiles
                .iter()
                .any(|tile| tile.tile_spec == MapTileSpec::Jukebox)
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
            jukebox_player: None,
            jukebox_ticks: 0,
            has_jukebox,
            wedding: None,
            wedding_ticks: 0,
            evacuate_ticks: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::AcceptGuildCreationRequest {
                player_id,
                invitee_player_id,
            } => self.accept_guild_creation_request(player_id, invitee_player_id),
            Command::AcceptTradeRequest {
                player_id,
                target_player_id,
            } => self.accept_trade_request(player_id, target_player_id).await,
            Command::AcceptWeddingRequest { player_id } => self.accept_wedding_request(player_id),
            Command::AddChestItem { player_id, item } => self.add_chest_item(player_id, item).await,
            Command::AddLockerItem { player_id, item } => {
                self.add_locker_item(player_id, item).await
            }
            Command::AddTradeItem { player_id, item } => self.add_trade_item(player_id, item).await,
            Command::AgreeTrade { player_id } => self.accept_trade(player_id).await,
            Command::Attack {
                target_player_id,
                direction,
                timestamp,
            } => self.attack(target_player_id, direction, timestamp).await,

            Command::BuyItem {
                player_id,
                npc_index,
                item,
            } => self.buy_item(player_id, npc_index, item),

            Command::BuyHaircut {
                player_id,
                npc_index,
                hair_style,
                hair_color,
            } => self.buy_haircut(player_id, npc_index, hair_style, hair_color),

            Command::CancelTrade {
                player_id,
                partner_player_id,
            } => self.cancel_trade(player_id, partner_player_id),

            Command::CastSpell { player_id, target } => self.cast_spell(player_id, target).await,

            Command::CraftItem {
                player_id,
                npc_index,
                item_id,
            } => self.craft_item(player_id, npc_index, item_id),

            Command::CreateBoardPost {
                player_id,
                subject,
                body,
            } => self.create_board_post(player_id, subject, body).await,

            Command::FinishGuildCreation {
                player_id,
                member_ids,
                guild_tag,
                guild_name,
            } => self.finish_guild_creation(player_id, member_ids, guild_tag, guild_name),

            Command::DepositGold {
                player_id,
                session_id,
                amount,
            } => self.deposit_gold(player_id, session_id, amount).await,

            Command::DepositGuildGold { player_id, amount } => {
                self.deposit_guild_gold(player_id, amount)
            }

            Command::DisagreeTrade { player_id } => self.unaccept_trade(player_id).await,

            Command::DivorcePartner { player_id } => self.divorce_partner(player_id),

            Command::DropItem {
                target_player_id,
                item,
                coords,
            } => self.drop_item(target_player_id, item, coords).await,

            Command::EffectOnCoord { coords, effect_id } => {
                self.effect_on_coords(&[coords], effect_id)
            }

            Command::EffectOnPlayer {
                player_id,
                effect_id,
            } => self.effect_on_players(&[player_id], effect_id),

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

            Command::GetNpcIdForIndex {
                npc_index,
                respond_to,
            } => match self.npcs.get(&npc_index) {
                Some(npc) => {
                    let _ = respond_to.send(Some(npc.id));
                }
                None => {
                    let _ = respond_to.send(None);
                }
            },

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

            Command::AwardExperience { player_id, amount } => {
                self.award_experience(player_id, amount)
            }

            Command::GiveItem {
                target_player_id,
                item_id,
                amount,
            } => self.give_item(target_player_id, item_id, amount),

            Command::GiveKarma { player_id, amount } => self.give_karma(player_id, amount),

            Command::RemoveKarma { player_id, amount } => self.remove_karma(player_id, amount),

            Command::LoseItem {
                player_id,
                item_id,
                amount,
            } => self.lose_item(player_id, item_id, amount),

            Command::JoinGuild {
                player_id,
                recruiter_id,
                guild_tag,
                guild_name,
                guild_rank_string,
            } => self.join_guild(
                player_id,
                recruiter_id,
                guild_tag,
                guild_name,
                guild_rank_string,
            ),

            Command::JukeboxTimer => self.jukebox_timer(),

            Command::JunkItem {
                target_player_id,
                item_id,
                amount,
            } => self.junk_item(target_player_id, item_id, amount).await,

            Command::KickFromGuild { player_id } => self.kick_from_guild(player_id),

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

            Command::LeaveGuild { player_id } => self.leave_guild(player_id),

            Command::LevelStat { player_id, stat_id } => self.level_stat(player_id, stat_id),

            Command::OpenBank {
                player_id,
                npc_index,
            } => self.open_bank(player_id, npc_index).await,

            Command::OpenBarber {
                player_id,
                npc_index,
                session_id,
            } => self.open_barber(player_id, npc_index, session_id),

            Command::OpenBoard {
                player_id,
                board_id,
            } => self.open_board(player_id, board_id),

            Command::OpenChest { player_id, coords } => self.open_chest(player_id, coords),

            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords),

            Command::OpenGuildMaster {
                player_id,
                npc_index,
            } => self.open_guild_master(player_id, npc_index),

            Command::OpenInn {
                player_id,
                npc_index,
            } => self.open_inn(player_id, npc_index).await,

            Command::OpenJukebox { player_id } => self.open_jukebox(player_id),

            Command::OpenLaw {
                player_id,
                npc_index,
                session_id,
            } => self.open_law(player_id, npc_index, session_id),

            Command::OpenLocker { player_id } => self.open_locker(player_id),

            Command::OpenPriest {
                player_id,
                npc_index,
                session_id,
            } => self.open_priest(player_id, npc_index, session_id),

            Command::OpenShop {
                player_id,
                npc_index,
                session_id,
            } => self.open_shop(player_id, npc_index, session_id),

            Command::OpenSkillMaster {
                player_id,
                npc_index,
            } => self.open_skill_master(player_id, npc_index).await,

            Command::PlayJukeboxTrack {
                player_id,
                track_id,
            } => self.play_jukebox_track(player_id, track_id),

            Command::RecoverNpcs => self.recover_npcs().await,

            Command::RecoverPlayers => self.recover_players().await,

            Command::RemoveBoardPost { player_id, post_id } => {
                self.remove_board_post(player_id, post_id).await
            }

            Command::RemoveCitizenship { player_id } => self.remove_citizenship(player_id).await,

            Command::RemoveTradeItem { player_id, item_id } => {
                self.remove_trade_item(player_id, item_id).await
            }

            Command::ReplyToQuestNpc {
                player_id,
                npc_index,
                quest_id,
                session_id,
                action_id,
            } => self.reply_to_quest_npc(player_id, npc_index, quest_id, session_id, action_id),

            Command::RequestCitizenship {
                player_id,
                session_id,
                answers,
            } => {
                self.request_citizenship(player_id, session_id, answers)
                    .await
            }

            Command::Reload { file, file_size } => self.reload(file, file_size),

            Command::RequestDivorce {
                player_id,
                npc_index,
                name,
            } => self.request_divorce(player_id, npc_index, name),

            Command::RequestWedding {
                player_id,
                npc_index,
                name,
            } => self.request_wedding(player_id, npc_index, name),

            Command::RequestMarriageApproval {
                player_id,
                npc_index,
                name,
            } => self.request_marriage_approval(player_id, npc_index, name),

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

            Command::RequestToJoinGuild {
                player_id,
                guild_tag,
                recruiter_name,
            } => self.request_to_join_guild(player_id, guild_tag, recruiter_name),

            Command::RequestTrade {
                player_id,
                target_player_id,
            } => self.request_trade(player_id, target_player_id),

            Command::ResetCharacter {
                player_id,
                session_id,
            } => self.reset_character(player_id, session_id).await,

            Command::Save { respond_to } => self.save(respond_to).await,

            Command::SayIDo { player_id } => self.say_i_do(player_id),

            Command::SellItem {
                player_id,
                npc_index,
                item,
            } => self.sell_item(player_id, npc_index, item),

            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message),

            Command::SendGuildCreateRequests {
                leader_player_id,
                guild_identity,
            } => self.send_guild_create_requests(leader_player_id, guild_identity),

            Command::Serialize { respond_to } => {
                self.serialize(respond_to);
            }

            Command::SetClass {
                player_id,
                class_id,
            } => self.set_class(player_id, class_id),

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

            Command::StartEvacuate => {
                if self.evacuate_ticks.is_some() {
                    self.evacuate_ticks = None;
                } else {
                    self.evacuate_ticks = Some(SETTINGS.evacuate.timer_seconds);
                }
            }

            Command::SpawnItems => self.spawn_items().await,

            Command::SpawnNpcs => self.spawn_npcs().await,

            Command::TalkToQuestNpc {
                player_id,
                npc_index,
                quest_id,
                session_id,
            } => self.talk_to_quest_npc(player_id, npc_index, quest_id, session_id),

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

            Command::TimedWedding => self.timed_wedding(),

            Command::TimedEvacuate => self.timed_evacuate(),

            Command::ToggleHidden { player_id } => self.toggle_hidden(player_id),

            Command::ActNpcs => self.act_npcs(),

            Command::Unequip {
                player_id,
                item_id,
                sub_loc,
            } => self.unequip(player_id, item_id, sub_loc),

            Command::UpdateGuildRank {
                player_id,
                rank,
                rank_str,
            } => self.update_guild_rank(player_id, rank, rank_str),

            Command::UpgradeLocker { player_id } => self.upgrade_locker(player_id),

            Command::UseItem { player_id, item_id } => self.use_item(player_id, item_id).await,

            Command::ViewBoardPost { player_id, post_id } => {
                self.view_board_post(player_id, post_id).await
            }

            Command::ViewQuestHistory { player_id } => self.view_quest_history(player_id),

            Command::ViewQuestProgress { player_id } => self.view_quest_progress(player_id),

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
            Command::FindPlayer { player_id, name } => self.find_player(player_id, name),
            Command::RequestNpcs {
                player_id,
                npc_indexes,
            } => self.request_npcs(player_id, npc_indexes),
            Command::RequestPlayers {
                player_id,
                player_ids,
            } => self.request_players(player_id, player_ids),
            Command::RequestPlayersAndNpcs {
                player_id,
                player_ids,
                npc_indexes,
            } => self.request_players_and_npcs(player_id, player_ids, npc_indexes),
            Command::RequestRefresh { player_id } => self.request_refresh(player_id),
            Command::Quake { magnitude } => self.quake(magnitude),
        }
    }
}
