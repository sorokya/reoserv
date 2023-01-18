use eo::pubs::{DropNpc, EnfNpc, TalkNpc};

#[derive(Debug)]
pub struct NpcData {
    pub npc_record: EnfNpc,
    pub drop_record: Option<DropNpc>,
    pub talk_record: Option<TalkNpc>,
}
