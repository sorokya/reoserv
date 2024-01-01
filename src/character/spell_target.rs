#[derive(Debug)]
pub enum SpellTarget {
    Player,
    Group,
    OtherPlayer(i32),
    Npc(i32),
}
