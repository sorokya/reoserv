#[derive(Debug)]
pub struct Wedding {
    pub player_id: i32,
    pub partner_id: i32,
    pub npc_index: i32,
    pub state: WeddingState,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WeddingState {
    Requested,
    Accepted,
    PriestDialog1,
    PriestDialog2,
    PriestDoYouPartner,
    AskPartner,
    WaitingForPartner,
    PartnerAgrees,
    PriestDoYouPlayer,
    AskPlayer,
    WaitingForPlayer,
    PlayerAgrees,
    PriestDialog3,
    PriestDialog4,
    Hearts,
    PriestDialog5AndConfetti,
    Done,
}
