use eo::data::pubs::{NPCRecord, DropNPCRecord, TalkRecord};

#[derive(Debug)]
pub struct NpcData {
    pub npc_record: NPCRecord,
    pub drop_record: Option<DropNPCRecord>,
    pub talk_record: Option<TalkRecord>,
}
