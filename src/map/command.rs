use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort, EOThree},
    protocol::{
        server::range, Coords, Direction, Emote, Item, NearbyInfo, ShortItem, StatId, WarpAnimation,
    },
};
use tokio::sync::oneshot;

use crate::character::{Character, SpellTarget};

#[derive(Debug)]
pub enum Command {
    AcceptTradeRequest {
        player_id: EOShort,
        target_player_id: EOShort,
    },
    AddChestItem {
        player_id: EOShort,
        item: Item,
    },
    AddLockerItem {
        player_id: EOShort,
        item: Item,
    },
    Attack {
        target_player_id: EOShort,
        direction: Direction,
        timestamp: EOThree,
    },
    BuyItem {
        player_id: EOShort,
        item: Item,
        session_id: EOShort,
    },
    CancelTrade {
        player_id: EOShort,
    },
    CastSpell {
        player_id: EOShort,
        target: SpellTarget,
    },
    CraftItem {
        player_id: EOShort,
        item_id: EOShort,
        session_id: EOShort,
    },
    CreateBoardPost {
        player_id: EOShort,
        subject: String,
        body: String,
    },
    DepositGold {
        player_id: EOShort,
        session_id: EOThree,
        amount: EOInt,
    },
    DropItem {
        target_player_id: EOShort,
        item: ShortItem,
        coords: Coords,
    },
    Emote {
        target_player_id: EOShort,
        emote: Emote,
    },
    Enter {
        character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    },
    Equip {
        player_id: EOShort,
        item_id: EOShort,
        sub_loc: EOChar,
    },
    Face {
        target_player_id: EOShort,
        direction: Direction,
    },
    ForgetSkill {
        player_id: EOShort,
        skill_id: EOShort,
        session_id: EOShort,
    },
    GetCharacter {
        player_id: EOShort,
        respond_to: oneshot::Sender<Option<Box<Character>>>,
    },
    GetDimensions {
        respond_to: oneshot::Sender<(EOChar, EOChar)>,
    },
    GetItem {
        target_player_id: EOShort,
        item_index: EOShort,
    },
    GetMapInfo {
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
        respond_to: oneshot::Sender<range::Reply>,
    },
    GetNearbyInfo {
        target_player_id: EOShort,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    GetRelogCoords {
        respond_to: oneshot::Sender<Option<Coords>>,
    },
    GetRidAndSize {
        respond_to: oneshot::Sender<([EOShort; 2], EOInt)>,
    },
    GiveItem {
        target_player_id: EOShort,
        item_id: EOShort,
        amount: EOInt,
    },
    HasPlayer {
        player_id: EOShort,
        respond_to: oneshot::Sender<bool>,
    },
    JunkItem {
        target_player_id: EOShort,
        item_id: EOShort,
        amount: EOInt,
    },
    LearnSkill {
        player_id: EOShort,
        spell_id: EOShort,
        session_id: EOShort,
    },
    Leave {
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<Character>,
    },
    LevelStat {
        player_id: EOShort,
        stat_id: StatId,
    },
    OpenBank {
        player_id: EOShort,
        npc_index: EOChar,
    },
    OpenBoard {
        player_id: EOShort,
        board_id: EOShort,
    },
    OpenChest {
        player_id: EOShort,
        coords: Coords,
    },
    OpenDoor {
        target_player_id: EOShort, // TODO: rename to player_id
        door_coords: Coords,       // TODO: rename to coords
    },
    OpenLocker {
        player_id: EOShort,
    },
    OpenShop {
        player_id: EOShort,
        npc_index: EOChar,
    },
    OpenSkillMaster {
        player_id: EOShort,
        npc_index: EOChar,
    },
    RecoverNpcs,
    RecoverPlayers,
    RemoveBoardPost {
        player_id: EOShort,
        post_id: EOShort,
    },
    RequestPaperdoll {
        player_id: EOShort,
        target_player_id: EOShort,
    },
    RequestTrade {
        player_id: EOShort,
        target_player_id: EOShort,
    },
    ResetCharacter {
        player_id: EOShort,
        session_id: EOShort,
    },
    Save {
        respond_to: oneshot::Sender<()>,
    },
    SellItem {
        player_id: EOShort,
        item: Item,
        session_id: EOShort,
    },
    SendChatMessage {
        target_player_id: EOShort,
        message: String,
    },
    Serialize {
        respond_to: oneshot::Sender<Bytes>,
    },
    Sit {
        player_id: EOShort,
    },
    SitChair {
        player_id: EOShort,
        coords: Coords,
    },
    Stand {
        player_id: EOShort,
    },
    StartSpellChant {
        player_id: EOShort,
        spell_id: EOShort,
        timestamp: EOThree,
    },
    TakeChestItem {
        player_id: EOShort,
        item_id: EOShort,
    },
    TakeLockerItem {
        player_id: EOShort,
        item_id: EOShort,
    },
    TimedDoorClose,
    TimedDrain,
    TimedQuake,
    TimedSpikes,
    TimedWarpSuck,
    ToggleHidden {
        player_id: EOShort,
    },
    Unequip {
        player_id: EOShort,
        item_id: EOShort,
        sub_loc: EOChar,
    },
    UpgradeLocker {
        player_id: EOShort,
    },
    UseItem {
        player_id: EOShort,
        item_id: EOShort,
    },
    ViewBoardPost {
        player_id: EOShort,
        post_id: EOShort,
    },
    Walk {
        target_player_id: EOShort,
        direction: Direction,
        coords: Coords,
        timestamp: EOThree,
    },
    WithdrawGold {
        player_id: EOShort,
        session_id: EOThree,
        amount: EOInt,
    },
    SpawnItems,
    SpawnNpcs,
    ActNpcs,
}
