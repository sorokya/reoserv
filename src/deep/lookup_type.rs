#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LookupType {
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

impl Default for LookupType {
    fn default() -> Self {
        Self::Item
    }
}
