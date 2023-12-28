use eo::data::i32;

#[derive(Debug, Clone)]
pub struct Party {
    pub members: Vec<i32>,
    pub leader: i32,
}

impl Party {
    pub fn new(leader: i32, member: i32) -> Self {
        Self {
            leader,
            members: vec![leader, member],
        }
    }
}
