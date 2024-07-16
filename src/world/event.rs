use chrono::Utc;
use mail_send::mail_auth::zip::DateTime;

pub struct Event {
    pub character_id: i32,
    pub event_type: EventType,
}

pub enum EventType {
    LoggedIn,
    Disconnected,
    DroppedItem,
    PickedUpItem,
    JunkedItem,
    DepositedItem,
    WithdrewItem,
    TookChestItem,
    AddedChestItem,
    EnteredMap,
}
