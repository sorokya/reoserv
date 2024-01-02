use bytes::Bytes;
use eolib::protocol::{
    net::{
        client::{ByteCoords, StatId},
        server::{NearbyInfo, WarpEffect},
        Item, ThreeItem,
    },
    Coords, Direction, Emote,
};
use tokio::sync::oneshot;

use crate::{
    character::{Character, SpellTarget},
    player::PartyRequest,
};

#[derive(Debug)]
pub enum Command {
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
    AgreeTrade {
        player_id: i32,
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
        amount: i32,
    },
    DisagreeTrade {
        player_id: i32,
    },
    DropItem {
        target_player_id: i32,
        item: ThreeItem,
        coords: ByteCoords,
    },
    Emote {
        target_player_id: i32,
        emote: Emote,
    },
    Enter {
        character: Box<Character>,
        warp_animation: Option<WarpEffect>,
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
    FindPlayer {
        player_id: i32,
        name: String,
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
    GetNearbyInfo {
        target_player_id: i32,
        respond_to: oneshot::Sender<NearbyInfo>,
    },
    GetRelogCoords {
        respond_to: oneshot::Sender<Option<Coords>>,
    },
    GetRidAndSize {
        respond_to: oneshot::Sender<([i32; 2], i32)>,
    },
    GiveItem {
        target_player_id: i32,
        item_id: i32,
        amount: i32,
    },
    JukeboxTimer,
    JunkItem {
        target_player_id: i32,
        item_id: i32,
        amount: i32,
    },
    LearnSkill {
        player_id: i32,
        spell_id: i32,
        session_id: i32,
    },
    Leave {
        player_id: i32,
        warp_animation: Option<WarpEffect>,
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
        door_coords: Coords,   // TODO: rename to coords
    },
    OpenInn {
        player_id: i32,
        npc_index: i32,
    },
    OpenJukebox {
        player_id: i32,
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
    PlayJukeboxTrack {
        player_id: i32,
        track_id: i32,
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
    RequestNpcs {
        player_id: i32,
        npc_indexes: Vec<i32>,
    },
    RequestPaperdoll {
        player_id: i32,
        target_player_id: i32,
    },
    RequestPlayers {
        player_id: i32,
        player_ids: Vec<i32>,
    },
    RequestPlayersAndNpcs {
        player_id: i32,
        player_ids: Vec<i32>,
        npc_indexes: Vec<i32>,
    },
    RequestRefresh {
        player_id: i32,
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
        amount: i32,
    },
    SpawnItems,
    SpawnNpcs,
    ActNpcs,
}
