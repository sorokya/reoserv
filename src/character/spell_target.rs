use eo::data::{i32, EOShort};

#[derive(Debug)]
pub enum SpellTarget {
    Player,
    Group,
    OtherPlayer(EOShort),
    Npc(i32),
}
