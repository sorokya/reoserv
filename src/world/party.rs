use eo::data::EOShort;

#[derive(Debug)]
pub struct Party {
    pub members: Vec<EOShort>,
    pub leader: EOShort,
}

impl Party {
    pub fn new(leader: EOShort, member: EOShort) -> Self {
        Self {
            leader,
            members: vec![leader, member],
        }
    }
}
