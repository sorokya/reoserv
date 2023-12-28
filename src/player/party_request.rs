use eo::data::i32;

#[derive(Debug, Copy, Clone)]
pub enum PartyRequest {
    None,
    Invite(i32),
    Join(i32),
}
