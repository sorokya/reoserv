use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AccountConfigServerPacket {
    pub delay_time: i32,
    pub email_validation: bool,
}

impl EoSerialize for AccountConfigServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.delay_time = reader.get_short()?;
        packet.email_validation = reader.get_char()? == 1;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.delay_time)?;
        writer.add_char(if self.email_validation { 1 } else { 0 })?;
        Ok(())
    }
}
