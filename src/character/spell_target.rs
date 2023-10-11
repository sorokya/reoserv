use eo::data::{EOChar, EOShort};

#[derive(Debug)]
pub enum SpellTarget {
    Player,
    Group,
    OtherPlayer(EOShort),
    Npc(EOChar),
}
