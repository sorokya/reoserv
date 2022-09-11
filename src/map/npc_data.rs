use eo::data::pubs::{NPCRecord, DropNPCRecord, TalkRecord};

#[derive(Debug)]
pub struct NpcData {
    pub npc_record: NPCRecord,
    pub drop_record: Option<DropNPCRecord>,
    pub talk_record: Option<TalkRecord>,
}

impl NpcData {
    pub fn new(npc_record: NPCRecord, drop_record: Option<DropNPCRecord>, talk_record: Option<TalkRecord>) -> Self {
        Self {
            npc_record,
            drop_record,
            talk_record,
        }
    }
}
