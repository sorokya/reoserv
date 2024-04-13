#[derive(PartialEq, Eq)]
pub enum EquipResult {
    Failed,
    Equiped,
    Swapped(i32),
}
