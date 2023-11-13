use eo::data::EOShort;

#[derive(Debug, Copy, Clone)]
pub enum PartyRequest {
    None,
    Invite(EOShort),
    Join(EOShort),
}
