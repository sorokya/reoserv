use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort, EOThree},
    protocol::{
        server::range, Coords, Direction, Emote, Item, NearbyInfo, ShortItem, StatId, WarpAnimation,
    },
    pubs::EmfFile,
};
use mysql_async::Pool;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::character::{Character, SpellTarget};

use super::{Command, Map};

#[derive(Debug, Clone)]
pub struct MapHandle {
    tx: UnboundedSender<Command>,
}

impl MapHandle {
    pub fn new(id: EOShort, file_size: EOInt, pool: Pool, file: EmfFile) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let map = Map::new(id, file_size, file, pool, rx);
        let _ = tokio::task::Builder::new()
            .name(&format!("Map {}", id))
            .spawn(run_map(map));

        Self { tx }
    }

    pub fn add_chest_item(&self, player_id: EOShort, item: Item) {
        let _ = self.tx.send(Command::AddChestItem { player_id, item });
    }

    pub fn add_locker_item(&self, player_id: EOShort, item: Item) {
        let _ = self.tx.send(Command::AddLockerItem { player_id, item });
    }

    pub fn buy_item(&self, player_id: EOShort, item: Item, session_id: EOShort) {
        let _ = self.tx.send(Command::BuyItem {
            player_id,
            item,
            session_id,
        });
    }

    pub fn cast_spell(&self, player_id: EOShort, target: SpellTarget) {
        let _ = self.tx.send(Command::CastSpell { player_id, target });
    }

    pub fn craft_item(&self, player_id: EOShort, item_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::CraftItem {
            player_id,
            item_id,
            session_id,
        });
    }

    pub fn create_board_post(&self, player_id: EOShort, subject: String, body: String) {
        let _ = self.tx.send(Command::CreateBoardPost {
            player_id,
            subject,
            body,
        });
    }

    pub fn deposit_gold(&self, player_id: EOShort, session_id: EOThree, amount: EOInt) {
        let _ = self.tx.send(Command::DepositGold {
            player_id,
            session_id,
            amount,
        });
    }

    pub fn drop_item(&self, target_player_id: EOShort, item: ShortItem, coords: Coords) {
        let _ = self.tx.send(Command::DropItem {
            target_player_id,
            item,
            coords,
        });
    }

    pub fn emote(&self, target_player_id: u16, emote: Emote) {
        let _ = self.tx.send(Command::Emote {
            target_player_id,
            emote,
        });
    }

    pub async fn enter(&self, character: Box<Character>, warp_animation: Option<WarpAnimation>) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Enter {
            character,
            warp_animation,
            respond_to: tx,
        });
        rx.await.unwrap();
    }

    pub fn equip(&self, player_id: EOShort, item_id: EOShort, sub_loc: EOChar) {
        let _ = self.tx.send(Command::Equip {
            player_id,
            item_id,
            sub_loc,
        });
    }

    pub fn face(&self, target_player_id: EOShort, direction: Direction) {
        let _ = self.tx.send(Command::Face {
            target_player_id,
            direction,
        });
    }

    pub fn forget_skill(&self, player_id: EOShort, skill_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::ForgetSkill {
            player_id,
            skill_id,
            session_id,
        });
    }

    pub async fn get_character(&self, player_id: EOShort) -> Option<Box<Character>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter {
            player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    // TODO: use coords!
    pub async fn get_dimensions(&self) -> (EOChar, EOChar) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetDimensions { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn get_item(&self, target_player_id: EOShort, item_index: EOShort) {
        let _ = self.tx.send(Command::GetItem {
            item_index,
            target_player_id,
        });
    }

    pub async fn get_map_info(
        &self,
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
    ) -> range::Reply {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMapInfo {
            player_ids,
            npc_indexes,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_nearby_info(&self, target_player_id: EOShort) -> NearbyInfo {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNearbyInfo {
            target_player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_rid_and_size(&self) -> ([EOShort; 2], EOInt) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetRidAndSize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn give_item(&self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        let _ = self.tx.send(Command::GiveItem {
            target_player_id,
            item_id,
            amount,
        });
    }

    pub fn junk_item(&self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        let _ = self.tx.send(Command::JunkItem {
            target_player_id,
            item_id,
            amount,
        });
    }

    pub async fn has_player(&self, player_id: EOShort) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::HasPlayer {
            player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn learn_skill(&self, player_id: EOShort, spell_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::LearnSkill {
            player_id,
            spell_id,
            session_id,
        });
    }

    pub async fn leave(
        &self,
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
    ) -> Character {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Leave {
            target_player_id,
            warp_animation,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn level_stat(&self, player_id: EOShort, stat_id: StatId) {
        let _ = self.tx.send(Command::LevelStat { player_id, stat_id });
    }

    pub fn open_bank(&self, player_id: EOShort, npc_index: EOChar) {
        let _ = self.tx.send(Command::OpenBank {
            player_id,
            npc_index,
        });
    }

    pub fn open_board(&self, player_id: EOShort, board_id: EOShort) {
        let _ = self.tx.send(Command::OpenBoard {
            player_id,
            board_id,
        });
    }

    pub fn open_chest(&self, player_id: EOShort, coords: Coords) {
        let _ = self.tx.send(Command::OpenChest { player_id, coords });
    }

    pub fn open_door(&self, target_player_id: EOShort, door_coords: Coords) {
        let _ = self.tx.send(Command::OpenDoor {
            target_player_id,
            door_coords,
        });
    }

    pub fn open_locker(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::OpenLocker { player_id });
    }

    pub fn open_shop(&self, player_id: EOShort, npc_index: EOChar) {
        let _ = self.tx.send(Command::OpenShop {
            player_id,
            npc_index,
        });
    }

    pub fn open_skill_master(&self, player_id: EOShort, npc_index: EOChar) {
        let _ = self.tx.send(Command::OpenSkillMaster {
            player_id,
            npc_index,
        });
    }

    pub fn recover_npcs(&self) {
        let _ = self.tx.send(Command::RecoverNpcs);
    }

    pub fn recover_players(&self) {
        let _ = self.tx.send(Command::RecoverPlayers);
    }

    pub fn remove_board_post(&self, player_id: EOShort, post_id: EOShort) {
        let _ = self
            .tx
            .send(Command::RemoveBoardPost { player_id, post_id });
    }

    pub fn request_paperdoll(&self, player_id: EOShort, target_player_id: EOShort) {
        let _ = self.tx.send(Command::RequestPaperdoll {
            player_id,
            target_player_id,
        });
    }

    pub fn reset_character(&self, player_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::ResetCharacter {
            player_id,
            session_id,
        });
    }

    pub fn sell_item(&self, player_id: EOShort, item: Item, session_id: EOShort) {
        let _ = self.tx.send(Command::SellItem {
            player_id,
            item,
            session_id,
        });
    }

    pub fn sit(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::Sit { player_id });
    }

    pub fn sit_chair(&self, player_id: EOShort, coords: Coords) {
        let _ = self.tx.send(Command::SitChair { player_id, coords });
    }

    pub fn stand(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::Stand { player_id });
    }

    pub fn start_spell_chant(&self, player_id: EOShort, spell_id: EOShort, timestamp: EOThree) {
        let _ = self.tx.send(Command::StartSpellChant {
            player_id,
            spell_id,
            timestamp,
        });
    }

    pub async fn save(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Save { respond_to: tx });
        rx.await.unwrap();
    }

    pub fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        let _ = self.tx.send(Command::SendChatMessage {
            target_player_id,
            message,
        });
    }

    pub async fn serialize(&self) -> Bytes {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn spawn_items(&self) {
        let _ = self.tx.send(Command::SpawnItems);
    }

    pub fn spawn_npcs(&self) {
        let _ = self.tx.send(Command::SpawnNpcs);
    }

    pub fn take_chest_item(&self, player_id: EOShort, item_id: EOShort) {
        let _ = self.tx.send(Command::TakeChestItem { player_id, item_id });
    }

    pub fn take_locker_item(&self, player_id: EOShort, item_id: EOShort) {
        let _ = self.tx.send(Command::TakeLockerItem { player_id, item_id });
    }

    pub fn timed_quake(&self) {
        let _ = self.tx.send(Command::TimedQuake);
    }

    pub fn toggle_hidden(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::ToggleHidden { player_id });
    }

    pub fn act_npcs(&self) {
        let _ = self.tx.send(Command::ActNpcs);
    }

    pub fn unequip(&self, player_id: EOShort, item_id: EOShort, sub_loc: EOChar) {
        let _ = self.tx.send(Command::Unequip {
            player_id,
            item_id,
            sub_loc,
        });
    }

    pub fn upgrade_locker(&self, player_id: EOShort) {
        let _ = self.tx.send(Command::UpgradeLocker { player_id });
    }

    pub fn use_item(&self, player_id: EOShort, item_id: EOShort) {
        let _ = self.tx.send(Command::UseItem { player_id, item_id });
    }

    pub fn view_board_post(&self, player_id: EOShort, post_id: EOShort) {
        let _ = self.tx.send(Command::ViewBoardPost { player_id, post_id });
    }

    pub fn walk(
        &self,
        target_player_id: EOShort,
        direction: Direction,
        coords: Coords,
        timestamp: EOThree,
    ) {
        let _ = self.tx.send(Command::Walk {
            target_player_id,
            direction,
            coords,
            timestamp,
        });
    }

    pub fn withdraw_gold(&self, player_id: EOShort, session_id: EOThree, amount: EOInt) {
        let _ = self.tx.send(Command::WithdrawGold {
            player_id,
            session_id,
            amount,
        });
    }

    pub fn attack(&self, target_player_id: EOShort, direction: Direction, timestamp: EOThree) {
        let _ = self.tx.send(Command::Attack {
            target_player_id,
            direction,
            timestamp,
        });
    }
}

async fn run_map(mut map: Map) {
    loop {
        if let Some(command) = map.rx.recv().await {
            map.handle_command(command).await;
        }
    }
}
