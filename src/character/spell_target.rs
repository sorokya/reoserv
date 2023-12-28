use eo::data::{i32, i32};

#[derive(Debug)]
pub enum SpellTarget {
    Player,
    Group,
    OtherPlayer(i32),
    Npc(i32),
}
