#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum LookupType {
    #[default]
    Item,
    Npc,
    Unrecognized(i32),
}

impl From<i32> for LookupType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Item,
            2 => Self::Npc,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<LookupType> for i32 {
    fn from(value: LookupType) -> i32 {
        match value {
            LookupType::Item => 1,
            LookupType::Npc => 2,
            LookupType::Unrecognized(value) => value,
        }
    }
}
