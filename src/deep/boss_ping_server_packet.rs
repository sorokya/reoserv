use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct BossPingServerPacket {
    pub npc_index: i32,
    pub npc_id: i32,
    pub hp: i32,
    pub hp_percentage: i32,
    pub killed: bool,
}

impl BossPingServerPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for BossPingServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.npc_index = reader.get_short()?;
        packet.npc_id = reader.get_short()?;
        packet.hp = reader.get_three()?;
        packet.hp_percentage = reader.get_char()?;
        packet.killed = reader.get_char()? == 1;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.npc_index)?;
        writer.add_short(self.npc_id)?;
        writer.add_three(self.hp)?;
        writer.add_char(self.hp_percentage)?;
        writer.add_char(if self.killed { 1 } else { 0 })?;
        Ok(())
    }
}
