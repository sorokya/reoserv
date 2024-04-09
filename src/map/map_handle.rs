use bytes::Bytes;
use eolib::protocol::{
    map::Emf,
    net::{
        client::{ByteCoords, StatId},
        server::{NearbyInfo, WarpEffect},
        Item, ThreeItem,
    },
    Coords, Direction, Emote,
};
use mysql_async::Pool;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::{
    character::{Character, SpellTarget},
    player::PartyRequest,
    world::WorldHandle,
};

use super::{Command, Map};

#[derive(Debug, Clone)]
pub struct MapHandle {
    tx: UnboundedSender<Command>,
}

impl MapHandle {
    pub fn new(id: i32, file_size: i32, pool: Pool, file: Emf, world: WorldHandle) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let map = Map::new(id, file_size, file, pool, world, rx);
        tokio::spawn(run_map(map));

        Self { tx }
    }

    pub fn accept_guild_creation_request(&self, player_id: i32, invitee_player_id: i32) {
        let _ = self.tx.send(Command::AcceptGuildCreationRequest {
            player_id,
            invitee_player_id,
        });
    }

    pub fn accept_trade_request(&self, player_id: i32, target_player_id: i32) {
        let _ = self.tx.send(Command::AcceptTradeRequest {
            player_id,
            target_player_id,
        });
    }

    pub fn accept_wedding_request(&self, player_id: i32) {
        let _ = self.tx.send(Command::AcceptWeddingRequest { player_id });
    }

    pub fn add_chest_item(&self, player_id: i32, item: Item) {
        let _ = self.tx.send(Command::AddChestItem { player_id, item });
    }

    pub fn add_locker_item(&self, player_id: i32, item: Item) {
        let _ = self.tx.send(Command::AddLockerItem { player_id, item });
    }

    pub fn add_trade_item(&self, player_id: i32, item: Item) {
        let _ = self.tx.send(Command::AddTradeItem { player_id, item });
    }

    pub fn agree_trade(&self, player_id: i32) {
        let _ = self.tx.send(Command::AgreeTrade { player_id });
    }

    pub fn buy_item(&self, player_id: i32, npc_index: i32, item: Item) {
        let _ = self.tx.send(Command::BuyItem {
            player_id,
            npc_index,
            item,
        });
    }

    pub fn buy_haircut(&self, player_id: i32, npc_index: i32, hair_style: i32, hair_color: i32) {
        let _ = self.tx.send(Command::BuyHaircut {
            player_id,
            npc_index,
            hair_style,
            hair_color,
        });
    }

    pub fn cancel_trade(&self, player_id: i32, partner_player_id: i32) {
        let _ = self.tx.send(Command::CancelTrade {
            player_id,
            partner_player_id,
        });
    }

    pub fn cast_spell(&self, player_id: i32, target: SpellTarget) {
        let _ = self.tx.send(Command::CastSpell { player_id, target });
    }

    pub fn craft_item(&self, player_id: i32, npc_index: i32, item_id: i32) {
        let _ = self.tx.send(Command::CraftItem {
            player_id,
            npc_index,
            item_id,
        });
    }

    pub fn create_board_post(&self, player_id: i32, subject: String, body: String) {
        let _ = self.tx.send(Command::CreateBoardPost {
            player_id,
            subject,
            body,
        });
    }

    pub fn finish_guild_creation(
        &self,
        player_id: i32,
        member_ids: Vec<i32>,
        guild_tag: String,
        guild_name: String,
    ) {
        let _ = self.tx.send(Command::FinishGuildCreation {
            player_id,
            member_ids,
            guild_tag,
            guild_name,
        });
    }

    pub fn deposit_gold(&self, player_id: i32, npc_index: i32, amount: i32) {
        let _ = self.tx.send(Command::DepositGold {
            player_id,
            npc_index,
            amount,
        });
    }

    pub fn deposit_guild_gold(&self, player_id: i32, amount: i32) {
        let _ = self
            .tx
            .send(Command::DepositGuildGold { player_id, amount });
    }

    pub fn disagree_trade(&self, player_id: i32) {
        let _ = self.tx.send(Command::DisagreeTrade { player_id });
    }

    pub fn divorce_partner(&self, player_id: i32) {
        let _ = self.tx.send(Command::DivorcePartner { player_id });
    }

    pub fn drop_item(&self, target_player_id: i32, item: ThreeItem, coords: ByteCoords) {
        let _ = self.tx.send(Command::DropItem {
            target_player_id,
            item,
            coords,
        });
    }

    pub fn effect_on_player(&self, player_id: i32, effect_id: i32) {
        let _ = self.tx.send(Command::EffectOnPlayer {
            player_id,
            effect_id,
        });
    }

    pub fn effect_on_coord(&self, coords: Coords, effect_id: i32) {
        let _ = self.tx.send(Command::EffectOnCoord { coords, effect_id });
    }

    pub fn emote(&self, target_player_id: i32, emote: Emote) {
        let _ = self.tx.send(Command::Emote {
            target_player_id,
            emote,
        });
    }

    pub async fn enter(&self, character: Box<Character>, warp_animation: Option<WarpEffect>) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Enter {
            character,
            warp_animation,
            respond_to: tx,
        });
        rx.await.unwrap();
    }

    pub fn equip(&self, player_id: i32, item_id: i32, sub_loc: i32) {
        let _ = self.tx.send(Command::Equip {
            player_id,
            item_id,
            sub_loc,
        });
    }

    pub fn face(&self, target_player_id: i32, direction: Direction) {
        let _ = self.tx.send(Command::Face {
            target_player_id,
            direction,
        });
    }

    pub fn find_player(&self, player_id: i32, name: String) {
        let _ = self.tx.send(Command::FindPlayer { player_id, name });
    }

    pub fn forget_skill(&self, player_id: i32, skill_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::ForgetSkill {
            player_id,
            skill_id,
            session_id,
        });
    }

    pub async fn get_character(&self, player_id: i32) -> Option<Box<Character>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter {
            player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    // TODO: use coords!
    pub async fn get_dimensions(&self) -> (i32, i32) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetDimensions { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn get_item(&self, target_player_id: i32, item_index: i32) {
        let _ = self.tx.send(Command::GetItem {
            item_index,
            target_player_id,
        });
    }

    pub async fn get_nearby_info(&self, target_player_id: i32) -> NearbyInfo {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNearbyInfo {
            target_player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_npc_id_for_index(&self, npc_index: i32) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNpcIdForIndex {
            npc_index,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_relog_coords(&self) -> Option<Coords> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetRelogCoords { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_rid_and_size(&self) -> ([i32; 2], i32) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetRidAndSize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn award_experience(&self, player_id: i32, amount: i32) {
        let _ = self.tx.send(Command::AwardExperience { player_id, amount });
    }

    pub fn give_item(&self, target_player_id: i32, item_id: i32, amount: i32) {
        let _ = self.tx.send(Command::GiveItem {
            target_player_id,
            item_id,
            amount,
        });
    }

    pub fn give_karma(&self, player_id: i32, amount: i32) {
        let _ = self.tx.send(Command::GiveKarma { player_id, amount });
    }

    pub fn remove_karma(&self, player_id: i32, amount: i32) {
        let _ = self.tx.send(Command::RemoveKarma { player_id, amount });
    }

    pub fn reload(&self, file: Box<Emf>, file_size: i32) {
        let _ = self.tx.send(Command::Reload { file, file_size });
    }

    pub fn lose_item(&self, player_id: i32, item_id: i32, amount: i32) {
        let _ = self.tx.send(Command::LoseItem {
            player_id,
            item_id,
            amount,
        });
    }

    pub fn join_guild(
        &self,
        player_id: i32,
        recruiter_id: i32,
        guild_tag: String,
        guild_name: String,
        guild_rank_string: String,
    ) {
        let _ = self.tx.send(Command::JoinGuild {
            player_id,
            recruiter_id,
            guild_tag,
            guild_name,
            guild_rank_string,
        });
    }

    pub fn junk_item(&self, target_player_id: i32, item_id: i32, amount: i32) {
        let _ = self.tx.send(Command::JunkItem {
            target_player_id,
            item_id,
            amount,
        });
    }

    pub fn kick_from_guild(&self, player_id: i32) {
        let _ = self.tx.send(Command::KickFromGuild { player_id });
    }

    pub fn learn_skill(&self, player_id: i32, spell_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::LearnSkill {
            player_id,
            spell_id,
            session_id,
        });
    }

    pub async fn leave(
        &self,
        player_id: i32,
        warp_animation: Option<WarpEffect>,
        interact_player_id: Option<i32>,
    ) -> Character {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Leave {
            player_id,
            warp_animation,
            respond_to: tx,
            interact_player_id,
        });
        rx.await.unwrap()
    }

    pub fn leave_guild(&self, player_id: i32) {
        let _ = self.tx.send(Command::LeaveGuild { player_id });
    }

    pub fn level_stat(&self, player_id: i32, stat_id: StatId) {
        let _ = self.tx.send(Command::LevelStat { player_id, stat_id });
    }

    pub fn open_bank(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let _ = self.tx.send(Command::OpenBank {
            player_id,
            npc_index,
            session_id,
        });
    }

    pub fn open_barber(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let _ = self.tx.send(Command::OpenBarber {
            player_id,
            npc_index,
            session_id,
        });
    }

    pub fn open_board(&self, player_id: i32, board_id: i32) {
        let _ = self.tx.send(Command::OpenBoard {
            player_id,
            board_id,
        });
    }

    pub fn open_chest(&self, player_id: i32, coords: Coords) {
        let _ = self.tx.send(Command::OpenChest { player_id, coords });
    }

    pub fn open_door(&self, target_player_id: i32, door_coords: Coords) {
        let _ = self.tx.send(Command::OpenDoor {
            target_player_id,
            door_coords,
        });
    }

    pub fn open_guild_master(&self, player_id: i32, npc_index: i32) {
        let _ = self.tx.send(Command::OpenGuildMaster {
            player_id,
            npc_index,
        });
    }

    pub fn open_inn(&self, player_id: i32, npc_index: i32) {
        let _ = self.tx.send(Command::OpenInn {
            player_id,
            npc_index,
        });
    }

    pub fn open_jukebox(&self, player_id: i32) {
        let _ = self.tx.send(Command::OpenJukebox { player_id });
    }

    pub fn open_law(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let _ = self.tx.send(Command::OpenLaw {
            player_id,
            npc_index,
            session_id,
        });
    }

    pub fn open_locker(&self, player_id: i32) {
        let _ = self.tx.send(Command::OpenLocker { player_id });
    }

    pub fn open_priest(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let _ = self.tx.send(Command::OpenPriest {
            player_id,
            npc_index,
            session_id,
        });
    }

    pub fn open_shop(&self, player_id: i32, npc_index: i32, session_id: i32) {
        let _ = self.tx.send(Command::OpenShop {
            player_id,
            npc_index,
            session_id,
        });
    }

    pub fn open_skill_master(&self, player_id: i32, npc_index: i32) {
        let _ = self.tx.send(Command::OpenSkillMaster {
            player_id,
            npc_index,
        });
    }

    pub fn play_jukebox_track(&self, player_id: i32, track_id: i32) {
        let _ = self.tx.send(Command::PlayJukeboxTrack {
            player_id,
            track_id,
        });
    }

    pub fn recover_npcs(&self) {
        let _ = self.tx.send(Command::RecoverNpcs);
    }

    pub fn recover_players(&self) {
        let _ = self.tx.send(Command::RecoverPlayers);
    }

    pub fn remove_board_post(&self, player_id: i32, post_id: i32) {
        let _ = self
            .tx
            .send(Command::RemoveBoardPost { player_id, post_id });
    }

    pub fn remove_citizenship(&self, player_id: i32) {
        let _ = self.tx.send(Command::RemoveCitizenship { player_id });
    }

    pub fn remove_trade_item(&self, player_id: i32, item_id: i32) {
        let _ = self
            .tx
            .send(Command::RemoveTradeItem { player_id, item_id });
    }

    pub fn reply_to_quest_npc(
        &self,
        player_id: i32,
        npc_index: i32,
        quest_id: i32,
        session_id: i32,
        action_id: Option<i32>,
    ) {
        let _ = self.tx.send(Command::ReplyToQuestNpc {
            player_id,
            npc_index,
            quest_id,
            session_id,
            action_id,
        });
    }

    pub fn request_citizenship(&self, player_id: i32, session_id: i32, answers: [String; 3]) {
        let _ = self.tx.send(Command::RequestCitizenship {
            player_id,
            session_id,
            answers,
        });
    }

    pub fn request_divorce(&self, player_id: i32, npc_index: i32, name: String) {
        let _ = self.tx.send(Command::RequestDivorce {
            player_id,
            npc_index,
            name,
        });
    }

    pub fn request_wedding(&self, player_id: i32, npc_index: i32, name: String) {
        let _ = self.tx.send(Command::RequestWedding {
            player_id,
            npc_index,
            name,
        });
    }

    pub fn request_marriage_approval(&self, player_id: i32, npc_index: i32, name: String) {
        let _ = self.tx.send(Command::RequestMarriageApproval {
            player_id,
            npc_index,
            name,
        });
    }

    pub fn request_npcs(&self, player_id: i32, npc_indexes: Vec<i32>) {
        let _ = self.tx.send(Command::RequestNpcs {
            player_id,
            npc_indexes,
        });
    }

    pub fn request_paperdoll(&self, player_id: i32, target_player_id: i32) {
        let _ = self.tx.send(Command::RequestPaperdoll {
            player_id,
            target_player_id,
        });
    }

    pub fn request_players(&self, player_id: i32, player_ids: Vec<i32>) {
        let _ = self.tx.send(Command::RequestPlayers {
            player_id,
            player_ids,
        });
    }

    pub fn request_players_and_npcs(
        &self,
        player_id: i32,
        player_ids: Vec<i32>,
        npc_indexes: Vec<i32>,
    ) {
        let _ = self.tx.send(Command::RequestPlayersAndNpcs {
            player_id,
            player_ids,
            npc_indexes,
        });
    }

    pub fn request_refresh(&self, player_id: i32) {
        let _ = self.tx.send(Command::RequestRefresh { player_id });
    }

    pub fn request_sleep(&self, player_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::RequestSleep {
            player_id,
            session_id,
        });
    }

    pub fn party_request(&self, target_player_id: i32, request: PartyRequest) {
        let _ = self.tx.send(Command::PartyRequest {
            target_player_id,
            request,
        });
    }

    pub fn request_to_join_guild(&self, player_id: i32, guild_tag: String, recruiter_name: String) {
        let _ = self.tx.send(Command::RequestToJoinGuild {
            player_id,
            guild_tag,
            recruiter_name,
        });
    }

    pub fn request_trade(&self, player_id: i32, target_player_id: i32) {
        let _ = self.tx.send(Command::RequestTrade {
            player_id,
            target_player_id,
        });
    }

    pub fn reset_character(&self, player_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::ResetCharacter {
            player_id,
            session_id,
        });
    }

    pub fn sell_item(&self, player_id: i32, npc_index: i32, item: Item) {
        let _ = self.tx.send(Command::SellItem {
            player_id,
            npc_index,
            item,
        });
    }

    pub fn sit(&self, player_id: i32) {
        let _ = self.tx.send(Command::Sit { player_id });
    }

    pub fn sit_chair(&self, player_id: i32, coords: Coords) {
        let _ = self.tx.send(Command::SitChair { player_id, coords });
    }

    pub fn sleep(&self, player_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::Sleep {
            player_id,
            session_id,
        });
    }

    pub fn stand(&self, player_id: i32) {
        let _ = self.tx.send(Command::Stand { player_id });
    }

    pub fn start_spell_chant(&self, player_id: i32, spell_id: i32, timestamp: i32) {
        let _ = self.tx.send(Command::StartSpellChant {
            player_id,
            spell_id,
            timestamp,
        });
    }

    pub fn start_evacuate(&self) {
        let _ = self.tx.send(Command::StartEvacuate);
    }

    pub async fn save(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Save { respond_to: tx });
        rx.await.unwrap();
    }

    pub fn say_i_do(&self, player_id: i32) {
        let _ = self.tx.send(Command::SayIDo { player_id });
    }

    pub fn send_chat_message(&self, target_player_id: i32, message: String) {
        let _ = self.tx.send(Command::SendChatMessage {
            target_player_id,
            message,
        });
    }

    pub fn send_guild_create_requests(&self, leader_player_id: i32, guild_identity: String) {
        let _ = self.tx.send(Command::SendGuildCreateRequests {
            leader_player_id,
            guild_identity,
        });
    }

    pub async fn serialize(&self) -> Bytes {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn set_class(&self, player_id: i32, class_id: i32) {
        let _ = self.tx.send(Command::SetClass {
            player_id,
            class_id,
        });
    }

    pub fn spawn_items(&self) {
        let _ = self.tx.send(Command::SpawnItems);
    }

    pub fn spawn_npcs(&self) {
        let _ = self.tx.send(Command::SpawnNpcs);
    }

    pub fn talk_to_quest_npc(
        &self,
        player_id: i32,
        npc_index: i32,
        quest_id: i32,
        session_id: i32,
    ) {
        let _ = self.tx.send(Command::TalkToQuestNpc {
            player_id,
            npc_index,
            quest_id,
            session_id,
        });
    }

    pub fn take_chest_item(&self, player_id: i32, item_id: i32) {
        let _ = self.tx.send(Command::TakeChestItem { player_id, item_id });
    }

    pub fn take_locker_item(&self, player_id: i32, item_id: i32) {
        let _ = self.tx.send(Command::TakeLockerItem { player_id, item_id });
    }

    pub fn timed_arena(&self) {
        let _ = self.tx.send(Command::TimedArena);
    }

    pub fn jukebox_timer(&self) {
        let _ = self.tx.send(Command::JukeboxTimer);
    }

    pub fn timed_door_close(&self) {
        let _ = self.tx.send(Command::TimedDoorClose);
    }

    pub fn timed_drain(&self) {
        let _ = self.tx.send(Command::TimedDrain);
    }

    pub fn timed_quake(&self) {
        let _ = self.tx.send(Command::TimedQuake);
    }

    pub fn timed_spikes(&self) {
        let _ = self.tx.send(Command::TimedSpikes);
    }

    pub fn timed_warp_suck(&self) {
        let _ = self.tx.send(Command::TimedWarpSuck);
    }

    pub fn timed_wedding(&self) {
        let _ = self.tx.send(Command::TimedWedding);
    }

    pub fn timed_evacuate(&self) {
        let _ = self.tx.send(Command::TimedEvacuate);
    }

    pub fn toggle_hidden(&self, player_id: i32) {
        let _ = self.tx.send(Command::ToggleHidden { player_id });
    }

    pub fn act_npcs(&self) {
        let _ = self.tx.send(Command::ActNpcs);
    }

    pub fn unequip(&self, player_id: i32, item_id: i32, sub_loc: i32) {
        let _ = self.tx.send(Command::Unequip {
            player_id,
            item_id,
            sub_loc,
        });
    }

    pub fn update_guild_rank(&self, player_id: i32, rank: i32, rank_str: &str) {
        let _ = self.tx.send(Command::UpdateGuildRank {
            player_id,
            rank,
            rank_str: rank_str.to_owned(),
        });
    }

    pub fn upgrade_locker(&self, player_id: i32, npc_index: i32) {
        let _ = self.tx.send(Command::UpgradeLocker {
            player_id,
            npc_index,
        });
    }

    pub fn use_item(&self, player_id: i32, item_id: i32) {
        let _ = self.tx.send(Command::UseItem { player_id, item_id });
    }

    pub fn view_board_post(&self, player_id: i32, post_id: i32) {
        let _ = self.tx.send(Command::ViewBoardPost { player_id, post_id });
    }

    pub fn view_quest_history(&self, player_id: i32) {
        let _ = self.tx.send(Command::ViewQuestHistory { player_id });
    }

    pub fn view_quest_progress(&self, player_id: i32) {
        let _ = self.tx.send(Command::ViewQuestProgress { player_id });
    }

    pub fn walk(
        &self,
        target_player_id: i32,
        direction: Direction,
        coords: Coords,
        timestamp: i32,
    ) {
        let _ = self.tx.send(Command::Walk {
            target_player_id,
            direction,
            coords,
            timestamp,
        });
    }

    pub fn withdraw_gold(&self, player_id: i32, npc_index: i32, amount: i32) {
        let _ = self.tx.send(Command::WithdrawGold {
            player_id,
            npc_index,
            amount,
        });
    }

    pub fn attack(&self, target_player_id: i32, direction: Direction, timestamp: i32) {
        let _ = self.tx.send(Command::Attack {
            target_player_id,
            direction,
            timestamp,
        });
    }

    pub fn quake(&self, magnitude: i32) {
        let _ = self.tx.send(Command::Quake { magnitude });
    }
}

async fn run_map(mut map: Map) {
    loop {
        if let Some(command) = map.rx.recv().await {
            map.handle_command(command).await;
        }
    }
}
