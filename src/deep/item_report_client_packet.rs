use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ItemReportClientPacket {
    pub item_id: i32,
    pub title: String,
}

impl ItemReportClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for ItemReportClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);
        let mut packet = Self::new();
        packet.item_id = reader.get_short()?;
        reader.next_chunk()?;
        packet.title = reader.get_string()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.item_id)?;
        writer.add_byte(0xff);
        writer.add_string(&self.title);
        Ok(())
    }
}
