use bytes::Bytes;
use eo::{
    data::{i32, EOInt, i32, i32},
    protocol::{
        server::range, Coords, Direction, Emote, Item, NearbyInfo, ShortItem, StatId, WarpAnimation,
    },
};
use tokio::sync::oneshot;

use crate::{
    character::{Character, SpellTarget},
    player::PartyRequest,
};

#[derive(Debug)]
pub enum Command {
    AcceptTrade {
        player_id: i32,
    },
    AcceptTradeRequest {
        player_id: i32,
        target_player_id: i32,
    },
    AddChestItem {
        player_id: i32,
        item: Item,
    },
    AddLockerItem {
        player_id: i32,
        item: Item,
    },
    AddTradeItem {
        player_id: i32,
        item: Item,
    },
    Attack {
        target_player_id: i32,
        direction: Direction,
        timestamp: i32,
    },
    BuyItem {
        player_id: i32,
        item: Item,
        session_id: i32,
    },
    CancelTrade {
        player_id: i32,
        partner_player_id: i32,
    },
    CastSpell {
        player_id: i32,
        target: SpellTarget,
    },
    CraftItem {
        player_id: i32,
        item_id: i32,
        session_id: i32,
    },
    CreateBoardPost {
        player_id: i32,
        subject: String,
        body: String,
    },
    DepositGold {
        player_id: i32,
        session_id: i32,
        amount: EOInt,
    },
    DropItem {
        target_player_id: i32,
        item: ShortItem,
        coords: Coords,
    },
    Emote {
        target_player_id: i32,
        emote: Emote,
    },
    Enter {
        character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    },
    Equip {
        player_id: i32,
        item_id: i32,
        sub_loc: i32,
    },
    Face {
        target_player_id: i32,
        direction: Direction,
    },
    ForgetSkill {
        player_id: i32,
        skill_id: i32,
        session_id: i32,
    },
    GetCharacter {
        player_id: i32,
        respond_to: oneshot::Sender<Option<Box<Character>>>,
    },
    GetDimensions {
        respond_to: oneshot::Sender<(i32, i32)>,
    },
    GetItem {
        target_player_id: i32,
        item_index: i32,
    },
    GetMapInfo {
        player_ids: Vec<i32>,
        npc_indexes: Vec<i32>,
        respond_to: oneshot::Sender<range::Reply>,
    },
    GetNearbyInfo {
        target_player_id: i32,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    GetRelogCoords {
        respond_to: oneshot::Sender<Option<Coords>>,
    },
    GetRidAndSize {
        respond_to: oneshot::Sender<([i32; 2], EOInt)>,
    },
    GiveItem {
        target_player_id: i32,
        item_id: i32,
        amount: EOInt,
    },
    HasPlayer {
        player_id: i32,
        respond_to: oneshot::Sender<bool>,
    },
    JunkItem {
        target_player_id: i32,
        item_id: i32,
        amount: EOInt,
    },
    LearnSkill {
        player_id: i32,
        spell_id: i32,
        session_id: i32,
    },
    Leave {
        player_id: i32,
        warp_animation: Option<WarpAnimation>,
        interact_player_id: Option<i32>,
        respond_to: oneshot::Sender<Character>,
    },
    LevelStat {
        player_id: i32,
        stat_id: StatId,
    },
    OpenBank {
        player_id: i32,
        npc_index: i32,
    },
    OpenBoard {
        player_id: i32,
        board_id: i32,
    },
    OpenChest {
        player_id: i32,
        coords: Coords,
    },
    OpenDoor {
        target_player_id: i32, // TODO: rename to player_id
        door_coords: Coords,       // TODO: rename to coords
    },
    OpenInn {
        player_id: i32,
        npc_index: i32,
    },
    OpenLocker {
        player_id: i32,
    },
    OpenShop {
        player_id: i32,
        npc_index: i32,
    },
    OpenSkillMaster {
        player_id: i32,
        npc_index: i32,
    },
    RecoverNpcs,
    RecoverPlayers,
    RemoveBoardPost {
        player_id: i32,
        post_id: i32,
    },
    RemoveCitizenship {
        player_id: i32,
    },
    RemoveTradeItem {
        player_id: i32,
        item_id: i32,
    },
    RequestCitizenship {
        player_id: i32,
        session_id: i32,
        answers: [String; 3],
    },
    RequestPaperdoll {
        player_id: i32,
        target_player_id: i32,
    },
    RequestSleep {
        player_id: i32,
        session_id: i32,
    },
    PartyRequest {
        target_player_id: i32,
        request: PartyRequest,
    },
    RequestTrade {
        player_id: i32,
        target_player_id: i32,
    },
    ResetCharacter {
        player_id: i32,
        session_id: i32,
    },
    Save {
        respond_to: oneshot::Sender<()>,
    },
    SellItem {
        player_id: i32,
        item: Item,
        session_id: i32,
    },
    SendChatMessage {
        target_player_id: i32,
        message: String,
    },
    Serialize {
        respond_to: oneshot::Sender<Bytes>,
    },
    Sit {
        player_id: i32,
    },
    SitChair {
        player_id: i32,
        coords: Coords,
    },
    Sleep {
        player_id: i32,
        session_id: i32,
    },
    Stand {
        player_id: i32,
    },
    StartSpellChant {
        player_id: i32,
        spell_id: i32,
        timestamp: i32,
    },
    TakeChestItem {
        player_id: i32,
        item_id: i32,
    },
    TakeLockerItem {
        player_id: i32,
        item_id: i32,
    },
    TimedArena,
    TimedDoorClose,
    TimedDrain,
    TimedQuake,
    TimedSpikes,
    TimedWarpSuck,
    ToggleHidden {
        player_id: i32,
    },
    UnacceptTrade {
        player_id: i32,
    },
    Unequip {
        player_id: i32,
        item_id: i32,
        sub_loc: i32,
    },
    UpgradeLocker {
        player_id: i32,
    },
    UseItem {
        player_id: i32,
        item_id: i32,
    },
    ViewBoardPost {
        player_id: i32,
        post_id: i32,
    },
    Walk {
        target_player_id: i32,
        direction: Direction,
        coords: Coords,
        timestamp: i32,
    },
    WithdrawGold {
        player_id: i32,
        session_id: i32,
        amount: EOInt,
    },
    SpawnItems,
    SpawnNpcs,
    ActNpcs,
}
